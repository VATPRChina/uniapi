namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirportTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || (context.CurrentSegmentIndex == 1 && context.LastSegment?.Kind == RouteTokenKind.SPEED_AND_ALTITUDE)
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
