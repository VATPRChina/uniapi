using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class StarFallbackTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment is FixToken nextFix && nextFix.Fix is Airport
            && context.CurrentSegment is UnknownToken
            && context.CurrentSegmentIndex == context.SegmentCount - 2
            && context.LastSegment is FixToken;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment = new StarLegToken
        {
            Value = context.CurrentSegment.Value,
            Procedure = null,
        };
        return Task.FromResult(true);
    }
}
