namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirwayTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return (context.LastSegment?.Kind == RouteTokenKind.VHF
            || context.LastSegment?.Kind == RouteTokenKind.NDB
            || context.LastSegment?.Kind == RouteTokenKind.WAYPOINT
            ) && (context.NextSegment?.Kind == RouteTokenKind.VHF
                || context.NextSegment?.Kind == RouteTokenKind.NDB
                || context.NextSegment?.Kind == RouteTokenKind.WAYPOINT
            );
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return false;
        if (context.NextSegment == null) return false;
        var existsLeft = await navdataProvider.ExistsAirwayWithFix(
            context.CurrentSegment.Value,
            context.LastSegment.Value);
        var existsRight = await navdataProvider.ExistsAirwayWithFix(
            context.CurrentSegment.Value,
            context.NextSegment.Value);
        if (!existsLeft || !existsRight) return false;

        context.CurrentSegment.Kind = RouteTokenKind.AIRWAY;
        context.CurrentSegment.Id = Ulid.Empty;
        context.CurrentSegment.Value = context.CurrentSegment.Value;
        return true;
    }
}
