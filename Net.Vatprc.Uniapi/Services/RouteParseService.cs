using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

namespace Net.Vatprc.Uniapi.Services;

public class RouteParseService(DbNavdataAdapter navdata)
{
    protected DbNavdataAdapter Navdata => navdata;

    public async Task<IList<RouteToken>> ParseRouteAsync(string route)
    {
        var parser = new RouteParser(route, Navdata);
        await parser.ParseAllSegments();
        return parser.Tokens;
    }
}
