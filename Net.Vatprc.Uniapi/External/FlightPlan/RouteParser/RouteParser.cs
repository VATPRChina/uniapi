using System.Diagnostics;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;
using static Net.Vatprc.Uniapi.External.FlightPlan.INavdataProvider;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class RouteParser(string rawRoute, INavdataProvider navdata)
{
    protected readonly RouteLexer.RouteLexer Lexer = new(rawRoute, navdata);
    protected IList<FlightLeg> Legs = [];
    protected FlightFix? lastFixOverride = null;
    protected FlightFix LastFix => lastFixOverride ?? Legs.LastOrDefault()?.To
        ?? throw new InvalidOperationException("No last fix available.");

    public async Task<IList<FlightLeg>> Parse()
    {
        Legs = [];

        await Lexer.ParseAllSegments();

        for (var i = 0; i < Lexer.Tokens.Count; i++)
        {
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
                var airwayLegs = navdata.FindAirwayLegs(segment.Value);
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
                var path = BFSPath(fromFix, toFix, legLookup);
                Console.WriteLine($"Found path from {fromFix} to {toFix} with {path.Count} legs.");

                foreach (var leg in path)
                {
                    Console.WriteLine($"Adding leg from {leg.FromFixIdentifier} to {leg.ToFixIdentifier} ({leg.Ident})");
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
                            }
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
                        LegId = Ulid.Empty,
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

    protected IList<AirwayLeg> BFSPath(string from, string to, Dictionary<string, Dictionary<string, AirwayLeg>> legs)
    {
        var queue = new Queue<string>();
        var visited = new HashSet<string>();
        var parent = new Dictionary<string, string>();

        queue.Enqueue(from);
        visited.Add(from);

        while (queue.Count > 0)
        {
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

        Legs.Add(new FlightLeg
        {
            From = LastFix,
            To = fix,
            LegId = Ulid.Empty,
            LegIdentifier = "DCT",
            Type = FlightLeg.LegType.Direct,
        });
    }
}
