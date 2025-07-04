namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class StarTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment?.Kind == RouteTokenKind.AIRPORT;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return false;
        var proc = await navdataProvider.FindStar(context.CurrentSegment.Value, context.LastSegment.Value);
        if (proc == null) return false;

        context.CurrentSegment.Kind = RouteTokenKind.STAR;
        context.CurrentSegment.Id = proc.Id;
        context.CurrentSegment.Value = proc.Identifier;
        return true;
    }
}
