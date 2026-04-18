using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Services.FlightPlan;

public class EnrouteRouteComparator
{
    public static bool IsRouteMatchingExpected(
        IList<Leg> actual,
        IList<Leg> expected,
        ILogger<EnrouteRouteComparator> logger,
        CancellationToken ct = default)
    {
        var actualFirstEnroute = actual.FirstOrDefault(l => l is not ProcedureLeg && l.From is not Airport);
        int actualLeft = actualFirstEnroute != null ? actual.IndexOf(actualFirstEnroute) : 0;
        var actualLastEnroute = actual.LastOrDefault(l => l is not ProcedureLeg && l.To is not Airport);
        int actualRight = actualLastEnroute != null ? actual.IndexOf(actualLastEnroute) : actual.Count - 1;

        var expectedFirstMatching = expected.FirstOrDefault(i => actual[actualLeft].From == i.From);
        if (expectedFirstMatching == null)
        {
            logger.LogWarning("Expected route does not start with actual route segment: {ActualSegment}",
                actual[actualLeft].From.Name);
            return false;
        }
        int expectedIndex = expected.IndexOf(expectedFirstMatching);
        var expectedStart = expectedIndex;

        for (int i = actualLeft; i <= actualRight; i++)
        {
            if (ct.IsCancellationRequested)
            {
                logger.LogWarning("Route comparison cancelled.");
                return false;
            }
            if (expectedIndex >= expected.Count)
            {
                break;
            }
            if (expected[expectedIndex].From != actual[i].From
                || expected[expectedIndex].To != actual[i].To
                || (expected[expectedIndex] is AirwayLeg expectedAirwayLeg &&
                    expectedAirwayLeg.Identifier != (actual[i] as AirwayLeg)?.Identifier))
            {
                return false;
            }

            expectedIndex++;
        }

        if (expectedIndex - expectedStart < 1)
        {
            logger.LogWarning("Matched expected route is empty");
            return false;
        }

        logger.LogInformation("Route matches expected route from {From} to {To} for expected route segments: {FromIdent} to {ToIdent}",
            actual[actualLeft].From.Name, actual[actualRight].To.Name,
            expected[expectedStart].From.Name, expected[expectedIndex - 1].To.Name);

        return true;
    }
}
