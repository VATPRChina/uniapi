using System.Text.Json;
using Flurl;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.Models.Acdm;
using Serilog;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Validator;

public class Validator(Flight flight, List<FlightLeg> legs, INavdataProvider navdata)
{
    protected readonly Flight Flight = flight;
    protected readonly List<FlightLeg> Legs = legs;
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
        bool foundMatchingRoute = prefRteStr.Count == 0;
        foreach (var prefRte in prefRteStr)
        {
            var prefRteParsed = await new RouteParser.RouteParser(prefRte, Navdata).Parse();
            if (IsRouteMatchingExpected(Legs, prefRteParsed))
            {
                foundMatchingRoute = true;
                Logger.Information("Found matching route: {Route}", prefRte);
                break;
            }
            else
            {
                Logger.Information("Expected route {Expected} does not match.", prefRte);
            }
        }
        if (!foundMatchingRoute)
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

    protected static bool IsRouteMatchingExpected(List<FlightLeg> actual, List<FlightLeg> expected)
    {
        int actualLeft = actual.FindIndex(l => l.Type != FlightLeg.LegType.Sid && l.From.Type != FlightFix.FixType.Airport);
        if (actualLeft == -1) actualLeft = 0;
        int actualRight = actual.FindLastIndex(l => l.Type != FlightLeg.LegType.Star && l.To.Type != FlightFix.FixType.Airport);
        if (actualRight == -1) actualRight = actual.Count - 1;

        int expectedIndex = expected.FindIndex(i => actual[actualLeft].From.Identifier == i.From.Identifier);
        if (expectedIndex == -1)
        {
            Logger.Warning("Expected route does not start with actual route segment: {ActualSegment}",
                actual[actualLeft].From.Identifier);
            return false;
        }
        var expectedStart = expectedIndex;

        for (int i = actualLeft; i <= actualRight; i++)
        {
            if (expectedIndex >= expected.Count
                || expected[expectedIndex].From.Identifier != actual[i].From.Identifier
                || expected[expectedIndex].To.Identifier != actual[i].To.Identifier)
            {
                return false;
            }

            expectedIndex++;
        }

        Logger.Information("Route matches expected route from {From} to {To} for expected route segments: {FromIdent} to {ToIdent}",
            actual[actualLeft].From.Identifier, actual[actualRight].To.Identifier,
            expected[expectedStart].From.Identifier, expected[expectedIndex - 1].To.Identifier);

        return true;
    }
}
