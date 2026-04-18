namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class DctTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegment.Value == "DCT"
            && context.LastSegment is FixToken
            && context.NextSegment is FixToken;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.CurrentSegment.Value != "DCT")
        {
            throw new InvalidOperationException("DCT token handler can only resolve 'DCT' segment.");
        }

        context.CurrentSegment = new DirectLegToken
        {
            Value = "DCT",
        };
        return Task.FromResult(true);
    }
}
