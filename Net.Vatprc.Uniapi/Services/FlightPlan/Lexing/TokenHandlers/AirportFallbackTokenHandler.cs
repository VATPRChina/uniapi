namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirportFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || context.CurrentSegmentIndex == context.SegmentCount - 1;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.AIRPORT;
        context.CurrentSegment.Id = Ulid.Empty;
        return Task.FromResult(true);
    }
}
