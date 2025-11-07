namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

public interface ITokenHandler
{
    public bool NeedParseNextSegment => false;

    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider);

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider);
}
