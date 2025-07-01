using Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

namespace Net.Vatprc.Uniapi.Services;

public class RouteParseService(DbNavdataAdapter navdata)
{
    protected DbNavdataAdapter Navdata => navdata;

    public async Task<IList<FlightFix>> ParseRouteAsync(string route, string dep, string arr)
    {
        var parser = new RouteParser($"{dep} {route} {arr}", Navdata);
        return await parser.Parse();
    }
}
