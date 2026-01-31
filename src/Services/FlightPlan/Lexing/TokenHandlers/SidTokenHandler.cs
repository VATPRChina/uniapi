using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class SidTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment?.Kind == RouteTokenKind.AIRPORT;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return false;
        var proc = navdataProvider.FindSid(context.CurrentSegment.Value, context.LastSegment.Value);
        if (proc == null) return false;

        context.CurrentSegment.Kind = RouteTokenKind.SID;
        context.CurrentSegment.Id = proc.RecordId;
        context.CurrentSegment.Value = proc.Identifier;
        return true;
    }
}
