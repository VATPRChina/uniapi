namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public interface ITokenHandler
{
    public bool NeedParseNextSegment => false;

    public bool IsAllowed(IParseContext context, INavdataProvider navdataProvider);

    public Task Resolve(IParseContext context, INavdataProvider navdataProvider);
}
