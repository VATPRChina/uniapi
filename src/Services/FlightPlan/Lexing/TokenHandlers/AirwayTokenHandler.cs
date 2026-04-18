namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirwayTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment is FixToken
            && context.NextSegment is FixToken;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return false;
        if (context.NextSegment == null) return false;
        var existsLeft = await navdataProvider.ExistsAirwayWithFix(
            context.CurrentSegment.Value,
            context.LastSegment.Value);
        var existsRight = await navdataProvider.ExistsAirwayWithFix(
            context.CurrentSegment.Value,
            context.NextSegment.Value);
        if (!existsLeft || !existsRight) return false;

        context.CurrentSegment = new AirwayLegToken
        {
            Value = context.CurrentSegment.Value,
        };
        return true;
    }
}
