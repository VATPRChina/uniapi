namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirwayFallbackTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return (context.LastSegment is FixToken)
            && (context.NextSegment is FixToken)
            && context.CurrentSegment is UnknownToken
            && context.CurrentSegment.Value.Length >= 2
            && char.IsAsciiLetter(context.CurrentSegment.Value[0])
            && context.CurrentSegment.Value.Skip(1).All(char.IsAsciiDigit);
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment = new AirwayLegToken
        {
            Value = context.CurrentSegment.Value,
        };
        return Task.FromResult(true);
    }
}
