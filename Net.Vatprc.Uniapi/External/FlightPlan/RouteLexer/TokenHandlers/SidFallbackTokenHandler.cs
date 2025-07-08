namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class SidFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment?.Kind == RouteTokenKind.AIRPORT
            && context.CurrentSegment.Kind == RouteTokenKind.UNKNOWN
            && context.CurrentSegmentIndex == 1;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.SID;
        context.CurrentSegment.Id = Ulid.Empty;
        return Task.FromResult(true);
    }
}
