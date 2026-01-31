namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirportFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || (context.CurrentSegmentIndex == 1 && context.LastSegment?.Kind == RouteTokenKind.SPEED_AND_ALTITUDE)
            || context.CurrentSegmentIndex == context.SegmentCount - 1;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.AIRPORT;
        context.CurrentSegment.Id = string.Empty;
        return Task.FromResult(true);
    }
}
