using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class InitialSpeedAndAltitudeTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => false;

    protected const int MaxTokenLength = CruiseAltitudeParser.MaxTokenLength + CruiseSpeedParser.MaxTokenLength;
    protected const int MinTokenLength = CruiseAltitudeParser.MinTokenLength + CruiseSpeedParser.MinTokenLength;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
        && context.CurrentSegment.Value.Length >= MinTokenLength
        && context.CurrentSegment.Value.Length <= MaxTokenLength;
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var token = context.CurrentSegment.Value;

        if (!CruiseSpeedParser.TryParse(token, out var speed))
        {
            return Task.FromResult(false);
        }

        if (!CruiseAltitudeParser.TryParse(token[speed.Raw.Length..], out var _))
        {
            return Task.FromResult(false);
        }

        return Task.FromResult(true);
    }
}
