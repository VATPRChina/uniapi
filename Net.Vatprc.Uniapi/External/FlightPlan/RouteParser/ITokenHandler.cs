namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public interface ITokenHandler
{
    public bool NeedParseNextSegment => false;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider);

    public Task Resolve(ILexerContext context, INavdataProvider navdataProvider);
}
