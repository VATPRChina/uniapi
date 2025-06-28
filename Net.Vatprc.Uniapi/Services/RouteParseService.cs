using Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;

namespace Net.Vatprc.Uniapi.Services;

public class RouteParseService(DbNavdataAdapter navdata)
{
    protected DbNavdataAdapter Navdata => navdata;

    public async Task<IList<RouteToken>> ParseRouteAsync(string route, string dep, string arr)
    {
        var parser = new RouteLexer($"{dep} {route} {arr}", Navdata);
        await parser.ParseAllSegments();
        return parser.Tokens;
    }
}
