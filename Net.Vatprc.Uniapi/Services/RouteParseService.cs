using Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.External.FlightPlan.Validator;
using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services;

public class RouteParseService(DbNavdataAdapter navdata)
{
    protected DbNavdataAdapter Navdata => navdata;

    public async Task<IList<FlightLeg>> ParseRouteAsync(string route, string dep, string arr)
    {
        var parser = new RouteParser($"{dep} {route} {arr}", Navdata);
        return await parser.Parse();
    }

    public async Task<IList<Violation>> ValidateFlight(Flight flight, IList<FlightLeg> legs)
    {
        var validator = new Validator(flight, legs, Navdata);
        return await validator.Validate();
    }
}
