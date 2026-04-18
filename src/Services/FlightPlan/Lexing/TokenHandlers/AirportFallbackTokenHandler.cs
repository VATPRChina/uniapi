using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class AirportFallbackTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || (context.CurrentSegmentIndex == 1 && context.LastSegment is SpeedAndAltitudeToken)
            || context.CurrentSegmentIndex == context.SegmentCount - 1;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        context.CurrentSegment = new FixToken
        {
            Value = context.CurrentSegment.Value,
            Fix = new Airport(string.Empty, context.CurrentSegment.Value, 0, 0),
        };
        return Task.FromResult(true);
    }
}
