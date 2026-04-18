using Microsoft.Extensions.Caching.Memory;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class RouteParser
{
    protected readonly ILogger<RouteParser> Logger;
    protected readonly RouteLexer Lexer;
    protected IList<Leg> Legs = [];
    protected Fix? lastFixOverride = null;
    protected Fix LastFix => lastFixOverride ?? Legs.LastOrDefault()?.To
        ?? throw new InvalidOperationException("No last fix available.");
    protected IMemoryCache Cache;
    protected INavdataProvider Navdata;
    protected string RawRoute;

    public RouteParser(string rawRoute, INavdataProvider navdata, IMemoryCache cache, ILoggerFactory loggerFactory)
    {
        Logger = loggerFactory.CreateLogger<RouteParser>();
        RawRoute = rawRoute;
        Lexer = new(rawRoute, navdata, loggerFactory.CreateLogger<RouteLexer>(), loggerFactory);
        Navdata = navdata;
        Cache = cache;
    }

    public async Task<IList<Leg>> Parse(CancellationToken ct = default)
    {
        return await Cache.GetOrCreateAsync(RawRoute, async entry =>
        {
            Logger.LogInformation("Parsing route: {RawRoute}, since cache entry not found.", RawRoute);
            entry.SetSlidingExpiration(TimeSpan.FromHours(1));
            var parsed = await ParseWithoutCache(ct);
            if (!ct.IsCancellationRequested)
            {
                Logger.LogInformation("Parsed route: {RawRoute} with {LegCount} legs.", RawRoute, parsed.Count);
                return parsed;
            }
            else
            {
                Logger.LogWarning("Parsing of route {RawRoute} was cancelled.", RawRoute);
                throw new OperationCanceledException("Parsing was cancelled.", ct);
            }
        }) ?? throw new InvalidOperationException("Unexpected null for parse result.");
    }

    protected async Task<IList<Leg>> ParseWithoutCache(CancellationToken ct = default)
    {
        Legs = [];

        await Lexer.ParseAllSegments(ct);

        for (var i = 0; i < Lexer.Tokens.Count; i++)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Parsing cancelled.");
                return Legs;
            }

            var segment = Lexer.Tokens[i];

            if (lastFixOverride == null && Legs.Count == 0)
            {
                if (segment is FixToken fixToken)
                {
                    lastFixOverride = fixToken.Fix;
                }
                else
                {
                    throw new InvalidOperationException($"Unexpected token kind {segment.Kind} for initial fix.");
                }
            }
            else if (Legs.Count > 0)
            {
                lastFixOverride = null;
            }

            if (segment is SidLegToken)
            {
                continue; // TODO: Implement SID handling
            }
            else if (segment is StarLegToken)
            {
                continue; // TODO: Implement STAR handling
            }
            else if (segment is AirwayLegToken)
            {
                var legLookup = new Dictionary<string, Dictionary<string, AirwayLeg>>();
                var airwayLegs = Navdata.GetAirwayLegs(segment.Value);
                await foreach (var leg in airwayLegs)
                {
                    if (!legLookup.ContainsKey(leg.From.Identifier))
                    {
                        legLookup[leg.From.Identifier] = [];
                    }
                    legLookup[leg.From.Identifier][leg.To.Identifier] = leg;
                    if (!legLookup.ContainsKey(leg.To.Identifier))
                    {
                        legLookup[leg.To.Identifier] = [];
                    }
                    legLookup[leg.To.Identifier][leg.From.Identifier] = new AirwayLeg(leg.To, leg.From, leg.Identifier, leg.Direction switch
                    {
                        AirwayLeg.AirwayDirection.FORWARD => AirwayLeg.AirwayDirection.BACKWARD,
                        AirwayLeg.AirwayDirection.BACKWARD => AirwayLeg.AirwayDirection.FORWARD,
                        AirwayLeg.AirwayDirection.BOTH => AirwayLeg.AirwayDirection.BOTH,
                        _ => throw new InvalidOperationException($"Unexpected airway leg direction: {leg.Direction}"),
                    });
                }
                var fromFix = Lexer.Tokens[i - 1].Value;
                var toFix = Lexer.Tokens[i + 1].Value;
                var path = BFSPath(fromFix, toFix, legLookup, ct);
                Logger.LogInformation("Found path from {FromFix} to {ToFix} with {LegCount} legs.",
                    fromFix, toFix, path.Count);

                foreach (var leg in path)
                {
                    Logger.LogInformation(
                        "Adding leg from {From.Identifier} to {To.Identifier} ({Ident})",
                        leg.From.Identifier, leg.To.Identifier, leg.Identifier);
                    Legs.Add(leg);
                }
            }
            else if (segment is FixToken fixToken)
            {
                HandleWaypoint(fixToken);
            }
            else if (segment is SpeedAndAltitudeToken)
            {
                continue; // TODO: Implement STAR handling
            }
            else
            {
                throw new InvalidOperationException($"Unexpected token kind: {segment.Kind}");
            }
        }

        return Legs;
    }

    protected IList<AirwayLeg> BFSPath(string from, string to, Dictionary<string, Dictionary<string, AirwayLeg>> legs, CancellationToken ct = default)
    {
        var queue = new Queue<string>();
        var visited = new HashSet<string>();
        var parent = new Dictionary<string, string>();

        queue.Enqueue(from);
        visited.Add(from);

        while (queue.Count > 0)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("BFS path search cancelled.");
                return [];
            }

            var current = queue.Dequeue();

            if (current == to)
            {
                break;
            }

            if (!legs.ContainsKey(current))
            {
                continue;
            }

            foreach (var next in legs[current].Keys)
            {
                if (!visited.Contains(next))
                {
                    visited.Add(next);
                    parent[next] = current;
                    queue.Enqueue(next);
                }
            }
        }

        var path = new List<AirwayLeg>();
        var currentNode = to;

        while (currentNode != from && parent.ContainsKey(currentNode))
        {
            var leg = legs[parent[currentNode]][currentNode];
            path.Add(leg);
            currentNode = parent[currentNode];
        }

        path.Reverse();
        return path;
    }

    protected void HandleWaypoint(FixToken token)
    {
        if (LastFix.Latitude == token.Fix.Latitude && LastFix.Longitude == token.Fix.Longitude)
        {
            return;
        }

        Logger.LogInformation(
            "Adding direct leg from ({FromLat}, {FromLon}) to {ToIdentifier}",
            LastFix.Latitude, LastFix.Longitude, token.Value);

        Legs.Add(new DirectLeg(LastFix, token.Fix));
    }
}
