using Microsoft.Extensions.Caching.Memory;
using Net.Vatprc.Uniapi.External.FlightPlan.Lexing;
using Serilog;
using static Net.Vatprc.Uniapi.External.FlightPlan.INavdataProvider;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Parsing;

public class RouteParser
{
    protected readonly Serilog.ILogger Logger = Log.ForContext<RouteParser>();
    protected readonly RouteLexer Lexer;
    protected IList<FlightLeg> Legs = [];
    protected FlightFix? lastFixOverride = null;
    protected FlightFix LastFix => lastFixOverride ?? Legs.LastOrDefault()?.To
        ?? throw new InvalidOperationException("No last fix available.");
    protected IMemoryCache Cache;
    protected INavdataProvider Navdata;
    protected string RawRoute;

    public RouteParser(string rawRoute, INavdataProvider navdata, IMemoryCache cache)
    {
        RawRoute = rawRoute;
        Lexer = new(rawRoute, navdata);
        Navdata = navdata;
        Cache = cache;
    }

    public async Task<IList<FlightLeg>> Parse(CancellationToken ct = default)
    {
        return await Cache.GetOrCreateAsync(RawRoute, async entry =>
        {
            Logger.Information("Parsing route: {RawRoute}, since cache entry not found.", RawRoute);
            entry.SetSlidingExpiration(TimeSpan.FromHours(1));
            var parsed = await ParseWithoutCache(ct);
            if (!ct.IsCancellationRequested)
            {
                Logger.Information("Parsed route: {RawRoute} with {LegCount} legs.", RawRoute, parsed.Count);
                return parsed;
            }
            else
            {
                Logger.Warning("Parsing of route {RawRoute} was cancelled.", RawRoute);
                throw new OperationCanceledException("Parsing was cancelled.", ct);
            }
        }) ?? throw new InvalidOperationException("Unexpected null for parse result.");
    }

    protected async Task<IList<FlightLeg>> ParseWithoutCache(CancellationToken ct = default)
    {
        Legs = [];

        await Lexer.ParseAllSegments(ct);

        for (var i = 0; i < Lexer.Tokens.Count; i++)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.Warning("Parsing cancelled.");
                return Legs;
            }

            var segment = Lexer.Tokens[i];

            if (lastFixOverride == null && Legs.Count == 0)
            {
                lastFixOverride = new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = segment.Kind switch
                    {
                        RouteTokenKind.AIRPORT => FlightFix.FixType.Airport,
                        RouteTokenKind.VHF => FlightFix.FixType.Vhf,
                        RouteTokenKind.NDB => FlightFix.FixType.Ndb,
                        RouteTokenKind.WAYPOINT => FlightFix.FixType.Waypoint,
                        RouteTokenKind.GEO_COORD => FlightFix.FixType.GeoCoord,
                        RouteTokenKind.UNKNOWN => FlightFix.FixType.Unknown,
                        _ => throw new InvalidOperationException($"Unexpected token kind {segment.Kind} for initial fix."),
                    }
                };
            }
            else if (Legs.Count > 0)
            {
                lastFixOverride = null;
            }

            if (segment.Kind == RouteTokenKind.SID)
            {
                continue; // TODO: Implement SID handling
            }
            else if (segment.Kind == RouteTokenKind.STAR)
            {
                continue; // TODO: Implement STAR handling
            }
            else if (segment.Kind == RouteTokenKind.AIRWAY)
            {
                var legLookup = new Dictionary<string, Dictionary<string, AirwayLeg>>();
                var airwayLegs = Navdata.FindAirwayLegs(segment.Value);
                await foreach (var leg in airwayLegs)
                {
                    if (!legLookup.ContainsKey(leg.FromFixIdentifier))
                    {
                        legLookup[leg.FromFixIdentifier] = [];
                    }
                    legLookup[leg.FromFixIdentifier][leg.ToFixIdentifier] = leg;
                    if (!legLookup.ContainsKey(leg.ToFixIdentifier))
                    {
                        legLookup[leg.ToFixIdentifier] = [];
                    }
                    legLookup[leg.ToFixIdentifier][leg.FromFixIdentifier] = new AirwayLeg
                    {
                        Ident = leg.Ident,
                        FromFixIcaoCode = leg.ToFixIcaoCode,
                        FromFixIdentifier = leg.ToFixIdentifier,
                        FromFixId = leg.ToFixId,
                        FromFixType = leg.ToFixType,
                        ToFixIcaoCode = leg.FromFixIcaoCode,
                        ToFixIdentifier = leg.FromFixIdentifier,
                        ToFixId = leg.FromFixId,
                        ToFixType = leg.FromFixType,
                    };
                }
                var fromFix = Lexer.Tokens[i - 1].Value;
                var toFix = Lexer.Tokens[i + 1].Value;
                var path = BFSPath(fromFix, toFix, legLookup, ct);
                Logger.Information("Found path from {FromFix} to {ToFix} with {LegCount} legs.",
                    fromFix, toFix, path.Count);

                foreach (var leg in path)
                {
                    Logger.Information(
                        "Adding leg from {FromFixIdentifier}({FromFixId}) to {ToFixIdentifier}({ToFixId}) ({Ident})",
                        leg.FromFixIdentifier, leg.FromFixId, leg.ToFixIdentifier, leg.ToFixId, leg.Ident);
                    Legs.Add(new FlightLeg
                    {
                        From = new FlightFix
                        {
                            Id = leg.FromFixId,
                            Identifier = leg.FromFixIdentifier,
                            Type = leg.FromFixType switch
                            {
                                FixType.Waypoint => FlightFix.FixType.Waypoint,
                                FixType.Vhf => FlightFix.FixType.Vhf,
                                FixType.Ndb => FlightFix.FixType.Ndb,
                                _ => FlightFix.FixType.Unknown,
                            },
                        },
                        To = new FlightFix
                        {
                            Id = leg.ToFixId,
                            Identifier = leg.ToFixIdentifier,
                            Type = leg.ToFixType switch
                            {
                                FixType.Waypoint => FlightFix.FixType.Waypoint,
                                FixType.Vhf => FlightFix.FixType.Vhf,
                                FixType.Ndb => FlightFix.FixType.Ndb,
                                _ => FlightFix.FixType.Unknown,
                            },
                        },
                        LegId = (leg.FromFixId, leg.ToFixId),
                        LegIdentifier = leg.Ident,
                        Type = FlightLeg.LegType.Airway
                    });
                }
            }
            else if (segment.Kind == RouteTokenKind.AIRPORT)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.Airport,
                });
            }
            else if (segment.Kind == RouteTokenKind.VHF)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.Vhf,
                });
            }
            else if (segment.Kind == RouteTokenKind.NDB)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.Ndb,
                });
            }
            else if (segment.Kind == RouteTokenKind.WAYPOINT)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.Waypoint,
                });
            }
            else if (segment.Kind == RouteTokenKind.GEO_COORD)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.GeoCoord,
                });
            }
            else if (segment.Kind == RouteTokenKind.UNKNOWN)
            {
                HandleWaypoint(new FlightFix
                {
                    Id = segment.Id,
                    Identifier = segment.Value,
                    Type = FlightFix.FixType.Unknown,
                });
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
                Logger.Warning("BFS path search cancelled.");
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

    protected void HandleWaypoint(FlightFix fix)
    {
        if (LastFix.Identifier == fix.Identifier)
        {
            return;
        }

        Logger.Information(
            "Adding direct leg from {FromIdentifier}({FromId}) to {ToIdentifier}({ToId})",
            LastFix.Identifier, LastFix.Id, fix.Identifier, fix.Id);

        Legs.Add(new FlightLeg
        {
            From = LastFix,
            To = fix,
            LegId = null,
            LegIdentifier = "DCT",
            Type = FlightLeg.LegType.Direct,
        });
    }
}
