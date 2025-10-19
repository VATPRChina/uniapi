namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class StarTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment?.Kind == RouteTokenKind.AIRPORT;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.NextSegment == null) return false;
        var proc = await navdataProvider.FindStar(context.CurrentSegment.Value, context.NextSegment.Value);
        if (proc == null) return false;

        context.CurrentSegment.Kind = RouteTokenKind.STAR;
        context.CurrentSegment.Id = proc.Id;
        context.CurrentSegment.Value = proc.Identifier;
        return true;
    }
}
