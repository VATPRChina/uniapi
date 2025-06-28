namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;

public interface ITokenHandler
{
    public bool NeedParseNextSegment => false;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider);

    public Task Resolve(ILexerContext context, INavdataProvider navdataProvider);
}
