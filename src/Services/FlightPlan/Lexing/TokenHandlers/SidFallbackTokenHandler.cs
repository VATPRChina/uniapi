using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class SidFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment is FixToken lastFix && lastFix.Fix is Airport
            && context.CurrentSegment is UnknownToken
            && context.CurrentSegmentIndex == 1
            && context.NextSegment is FixToken;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment = new SidLegToken
        {
            Value = context.CurrentSegment.Value,
            Procedure = null,
        };
        return Task.FromResult(true);
    }
}
