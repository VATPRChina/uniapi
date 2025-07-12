using System.Text.Json;
using Flurl;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.Models.Acdm;
using Serilog;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Validator;

public class Validator(Flight flight, IList<FlightLeg> legs, INavdataProvider navdata)
{
    protected readonly Flight Flight = flight;
    protected readonly IList<FlightLeg> Legs = legs;
    protected readonly INavdataProvider Navdata = navdata;

    protected readonly List<Violation> Violations = [];

    protected static readonly Serilog.ILogger Logger = Log.ForContext<Validator>();

    public async Task<IList<Violation>> Validate()
    {
        if (!Flight.SupportRvsm)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                Type = Violation.ViolationType.NoRvsm,
            });
        }

        if (!Flight.SupportRnav1Equipment)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (!Flight.SupportRnav1Pbn)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (Flight.SupportRnpArWithoutRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.RnpArWithoutRf,
            });
        }

        if (Flight.SupportRnpArWithRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.RnpAr,
            });
        }

        var prefRteStr = await Navdata.GetRecommendedRoutes(Flight.Departure, Flight.Arrival);
        Logger.Information("Recommended routes for {Dep} to {Arr}: {Routes}",
            Flight.Departure, Flight.Arrival, prefRteStr);
        IList<IEnumerable<Violation>> violationsPerRoute = [];
        foreach (var prefRte in prefRteStr)
        {
            var prefRteParsed = await new RouteParser.RouteParser(prefRte, Navdata).Parse();
            var routeViolations = GetRouteDifferenceViolations(Legs, prefRteParsed, prefRte);
            if (routeViolations.Any())
            {
                Logger.Information("Route {Route} has violations: {Violations}",
                    prefRte, string.Join(", ", routeViolations.Select(v => v.ToString())));
                violationsPerRoute.Add(routeViolations);
            }
        }
        if (prefRteStr.Count > 0 && violationsPerRoute.Any(v => !v.Any()))
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Route,
                Type = Violation.ViolationType.NotRecommendedRoute,
                Param = string.Join(",", prefRteStr),
            });
        }

        foreach (var (leg, index) in Legs.Select((l, i) => (l, i)))
        {
            if (leg.LegId == null && leg.LegIdentifier == "DCT"
                && leg.From.Type != FlightFix.FixType.Airport
                && leg.To.Type != FlightFix.FixType.Airport)
            {
                Violations.Add(new Violation
                {
                    Field = Violation.FieldType.Route,
                    FieldParam = index,
                    Type = Violation.ViolationType.Direct,
                });
            }

            if (leg.LegId != null)
            {
                var (fromLegId, toLegId) = leg.LegId.Value;
                var fromLeg = await Navdata.GetAirwayFix(fromLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {fromLegId}");
                var toLeg = await Navdata.GetAirwayFix(toLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {toLegId}");

                Logger.Information("Validating airway leg {FromLeg} to {ToLeg} for flight {FlightId}",
                    fromLeg.FixIdentifier, toLeg.FixIdentifier, Flight.Id);
                Logger.Information("Restrictions: From leg: {FromLeg}, To leg: {ToLeg}", fromLeg.DirectionalRestriction, toLeg.DirectionalRestriction);
                Logger.Information("Sequence numbers: From leg: {FromSeq}, To leg: {ToSeq}",
                    fromLeg.SequenceNumber, toLeg.SequenceNumber);
                if (fromLeg.SequenceNumber <= toLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'B')
                {
                    Logger.Information("Violation found: From leg is backward.");
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.LegDirection,
                    });
                }
                if (toLeg.SequenceNumber <= fromLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'F')
                {
                    Logger.Information("Violation found: From leg is forward.");
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.LegDirection,
                    });
                }

                if ((fromLeg.FixIcaoCode.StartsWith("Z") &&
                        (fromLeg.AirwayIdentifier!.StartsWith("V") || fromLeg.AirwayIdentifier!.StartsWith("X")))
                    || (toLeg.FixIcaoCode.StartsWith("Z") &&
                            (toLeg.AirwayIdentifier!.StartsWith("V") || toLeg.AirwayIdentifier!.StartsWith("X"))))
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.AirwayRequireApproval,
                    });
                }
            }
        }

        return Violations;
    }

    protected static IEnumerable<Violation> GetRouteDifferenceViolations(IList<FlightLeg> route1, IList<FlightLeg> route2, string route2Raw)
    {
        IList<Violation> violations = [];

        int i = route1
            .TakeWhile(x => x.From.Identifier != route2.First().From.Identifier)
            .Count();
        int validLegs = 0;
        foreach (var leg2 in route2)
        {
            if (i >= route1.Count)
            {
                if (validLegs == 0)
                {
                    Logger.Information("Route {Route1} does not match preferred route {Route2}",
                        string.Join(" ", route1.Select(l => $"{l.From.Identifier}-{l.To.Identifier}")),
                        route2Raw);
                    violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        Type = Violation.ViolationType.NotRecommendedRoute,
                        Param = route2Raw,
                    });
                }
                break;
            }
            Logger.Information("Comparing leg {Leg1From}-{Leg1To} with preferred leg {Leg2From}-{Leg2To}",
                route1[i].From.Identifier, route1[i].To.Identifier,
                leg2.From.Identifier, leg2.To.Identifier);
            if (route1[i].From.Identifier != leg2.From.Identifier ||
                route1[i].To.Identifier != leg2.To.Identifier)
            {
                if (i + 1 == route1.Count)
                {
                    Logger.Information("Last leg {Leg1From}-{Leg1To} does not match preferred leg {Leg2From}-{Leg2To}",
                        route1[i].From.Identifier, route1[i].To.Identifier,
                        leg2.From.Identifier, leg2.To.Identifier);
                }
                else
                {
                    Logger.Information("Leg {Leg1From}-{Leg1To} does not match preferred leg {Leg2From}-{Leg2To}",
                        route1[i].From.Identifier, route1[i].To.Identifier,
                        leg2.From.Identifier, leg2.To.Identifier);
                    violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = i,
                        Type = Violation.ViolationType.NotRecommendedRoute,
                        Param = "".SetQueryParam("expected_from", leg2.From.Identifier)
                            .SetQueryParam("expected_to", leg2.To.Identifier)
                            .SetQueryParam("actual_from", route1[i].From.Identifier)
                            .SetQueryParam("actual_to", route1[i].To.Identifier)
                            .ToString(),
                    });
                }
            }
            else
            {
                validLegs += 1;
            }
            i += 1;
        }

        return violations;
    }
}
