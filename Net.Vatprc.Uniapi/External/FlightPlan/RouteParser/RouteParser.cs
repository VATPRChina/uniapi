using Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;
using static Net.Vatprc.Uniapi.External.FlightPlan.INavdataProvider;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class RouteParser(string rawRoute, INavdataProvider navdata)
{
    protected readonly RouteLexer.RouteLexer Lexer = new(rawRoute, navdata);

    public async Task<IList<FlightFix>> Parse()
    {
        var fixes = new List<FlightFix>();

        await Lexer.ParseAllSegments();

        for (var i = 0; i < Lexer.Tokens.Count; i++)
        {
            var segment = Lexer.Tokens[i];

            if (segment.Kind == RouteTokenKind.SID)
            {
                fixes.Last().ToNextLegAirwayId = segment.Id;
            }
            else if (segment.Kind == RouteTokenKind.STAR)
            {
                fixes.Last().ToNextLegAirwayId = segment.Id;
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
                    legLookup[leg.ToFixIdentifier][leg.FromFixIdentifier] = leg;
                }
                var fromFix = Lexer.Tokens[i - 1].Value;
                var toFix = Lexer.Tokens[i + 1].Value;
                var path = BFSPath(fromFix, toFix, legLookup);
                fixes.Last().ToNextLegAirwayId = path.First().FromFixId;
                foreach (var leg in path.Skip(1))
                {
                    fixes.Add(new FlightFix
                    {
                        ToNextLegAirwayId = leg.FromFixId,
                        FixIdentifier = leg.ToFixIdentifier,
                        FixId = leg.ToFixId, // TODO: set this to the actual fix ID if available
                        IsUnknown = false
                    });
                }
            }
            else if (segment.Kind == RouteTokenKind.AIRPORT)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = segment.Id,
                    IsUnknown = false
                });
            }
            else if (segment.Kind == RouteTokenKind.VHF)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = segment.Id,
                    IsUnknown = false
                });
            }
            else if (segment.Kind == RouteTokenKind.NDB)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = segment.Id,
                    IsUnknown = false
                });
            }
            else if (segment.Kind == RouteTokenKind.WAYPOINT)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = segment.Id,
                    IsUnknown = false
                });
            }
            else if (segment.Kind == RouteTokenKind.GEO_COORD)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = Ulid.Empty,
                    IsUnknown = true
                });
            }
            else if (segment.Kind == RouteTokenKind.UNKNOWN)
            {
                fixes.Add(new FlightFix
                {
                    ToNextLegAirwayId = Ulid.Empty,
                    FixIdentifier = segment.Value,
                    FixId = Ulid.Empty,
                    IsUnknown = true
                });
            }
            else
            {
                throw new InvalidOperationException($"Unexpected token kind: {segment.Kind}");
            }
        }

        return fixes;
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
}
