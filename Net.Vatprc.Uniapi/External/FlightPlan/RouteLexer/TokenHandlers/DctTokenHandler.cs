namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class DctTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegment.Value == "DCT"
            && (context.LastSegment?.Kind == RouteTokenKind.VHF
                || context.LastSegment?.Kind == RouteTokenKind.NDB
                || context.LastSegment?.Kind == RouteTokenKind.WAYPOINT
                || context.LastSegment?.Kind == RouteTokenKind.AIRPORT
                || context.LastSegment?.Kind == RouteTokenKind.GEO_COORD)
            && (context.NextSegment?.Kind == RouteTokenKind.VHF
                || context.NextSegment?.Kind == RouteTokenKind.NDB
                || context.NextSegment?.Kind == RouteTokenKind.WAYPOINT
                || context.NextSegment?.Kind == RouteTokenKind.AIRPORT
                || context.NextSegment?.Kind == RouteTokenKind.GEO_COORD
            );
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.CurrentSegment.Value != "DCT")
        {
            throw new InvalidOperationException("DCT token handler can only resolve 'DCT' segment.");
        }

        context.CurrentSegment.Kind = RouteTokenKind.AIRWAY;
        context.CurrentSegment.Id = Ulid.Empty;
        context.CurrentSegment.Value = context.CurrentSegment.Value;
        return Task.FromResult(true);
    }
}
