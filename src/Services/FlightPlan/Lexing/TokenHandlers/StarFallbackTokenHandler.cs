namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class StarFallbackTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment?.Kind == RouteTokenKind.AIRPORT
            && context.CurrentSegment.Kind == RouteTokenKind.UNKNOWN
            && context.CurrentSegmentIndex == context.SegmentCount - 2
            && (context.LastSegment?.Kind.IsFix() ?? false);
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.STAR;
        context.CurrentSegment.Id = string.Empty;
        return Task.FromResult(true);
    }
}
