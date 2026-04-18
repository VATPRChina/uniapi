using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class SidTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment is FixToken lastFix && lastFix.Fix is Airport;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return false;
        var proc = await navdataProvider.FindSid(context.CurrentSegment.Value, context.LastSegment.Value);
        if (proc == null) return false;

        context.CurrentSegment = new SidLegToken
        {
            Value = proc.Identifier,
            Procedure = proc,
        };
        return true;
    }
}
