using System.Text.RegularExpressions;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class AirwayFallbackTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return (context.LastSegment?.Kind == RouteTokenKind.VHF
                || context.LastSegment?.Kind == RouteTokenKind.NDB
                || context.LastSegment?.Kind == RouteTokenKind.WAYPOINT)
            && (context.NextSegment?.Kind == RouteTokenKind.VHF
                || context.NextSegment?.Kind == RouteTokenKind.NDB
                || context.NextSegment?.Kind == RouteTokenKind.WAYPOINT)
            && context.CurrentSegment.Kind == RouteTokenKind.UNKNOWN
            && new Regex("^[A-Za-z][0-9]+$").IsMatch(context.CurrentSegment.Value);
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment.Kind = RouteTokenKind.AIRWAY;
        context.CurrentSegment.Id = Ulid.Empty;
        return Task.FromResult(true);
    }
}
