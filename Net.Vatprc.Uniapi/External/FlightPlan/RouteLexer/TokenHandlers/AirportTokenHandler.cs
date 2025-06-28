namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class AirportTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || context.CurrentSegmentIndex == context.SegmentCount - 1;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var airport = await navdataProvider.FindAirport(context.CurrentSegment.Value);
        if (airport == null) return false;

        context.CurrentSegment.Kind = RouteTokenKind.AIRPORT;
        context.CurrentSegment.Value = airport.Identifier;
        context.CurrentSegment.Id = airport.Id;
        context.CurrentLon = airport.Longitude;
        context.CurrentLat = airport.Latitude;
        return true;
    }
}
