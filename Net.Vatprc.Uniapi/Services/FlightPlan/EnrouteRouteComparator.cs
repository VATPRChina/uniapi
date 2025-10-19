using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan;

public class EnrouteRouteComparator
{
    protected static readonly Serilog.ILogger Logger = Serilog.Log.ForContext<EnrouteRouteComparator>();

    public static bool IsRouteMatchingExpected(IList<FlightLeg> actual, IList<FlightLeg> expected, CancellationToken ct = default)
    {
        var actualFirstEnroute = actual.FirstOrDefault(l => l.Type != FlightLeg.LegType.Sid && l.From.Type != FlightFix.FixType.Airport);
        int actualLeft = actualFirstEnroute != null ? actual.IndexOf(actualFirstEnroute) : 0;
        var actualLastEnroute = actual.LastOrDefault(l => l.Type != FlightLeg.LegType.Star && l.To.Type != FlightFix.FixType.Airport);
        int actualRight = actualLastEnroute != null ? actual.IndexOf(actualLastEnroute) : actual.Count - 1;

        var expectedFirstMatching = expected.FirstOrDefault(i => actual[actualLeft].From.Identifier == i.From.Identifier);
        if (expectedFirstMatching == null)
        {
            Logger.Warning("Expected route does not start with actual route segment: {ActualSegment}",
                actual[actualLeft].From.Identifier);
            return false;
        }
        int expectedIndex = expected.IndexOf(expectedFirstMatching);
        var expectedStart = expectedIndex;

        for (int i = actualLeft; i <= actualRight; i++)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.Warning("Route comparison cancelled.");
                return false;
            }
            if (expectedIndex >= expected.Count)
            {
                break;
            }
            if (expected[expectedIndex].From.Identifier != actual[i].From.Identifier
                || expected[expectedIndex].To.Identifier != actual[i].To.Identifier
                || expected[expectedIndex].LegIdentifier != actual[i].LegIdentifier)
            {
                return false;
            }

            expectedIndex++;
        }

        if (expectedIndex - expectedStart < 1)
        {
            Logger.Warning("Matched expected route is empty");
            return false;
        }

        Logger.Information("Route matches expected route from {From} to {To} for expected route segments: {FromIdent} to {ToIdent}",
            actual[actualLeft].From.Identifier, actual[actualRight].To.Identifier,
            expected[expectedStart].From.Identifier, expected[expectedIndex - 1].To.Identifier);

        return true;
    }
}
