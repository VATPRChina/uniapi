namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class SidFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment?.Kind == RouteTokenKind.AIRPORT
            && context.CurrentSegment.Kind == RouteTokenKind.UNKNOWN
            && context.CurrentSegmentIndex == 1
            && (context.NextSegment?.Kind.IsFix() ?? false);
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.SID;
        context.CurrentSegment.Id = string.Empty;
        return Task.FromResult(true);
    }
}
