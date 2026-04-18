using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class StarTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment is FixToken nextFix && nextFix.Fix is Airport;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.NextSegment == null) return false;
        var proc = await navdataProvider.FindStar(context.CurrentSegment.Value, context.NextSegment.Value);
        if (proc == null) return false;

        context.CurrentSegment.Value = proc.Identifier;
        context.CurrentSegment = new StarLegToken
        {
            Value = proc.Identifier,
            Procedure = proc,
        };
        return true;
    }
}
