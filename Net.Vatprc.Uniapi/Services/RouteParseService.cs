using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.External.FlightPlan.Lexing;
using Net.Vatprc.Uniapi.External.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.External.FlightPlan.Validating;
using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services;

public class RouteParseService(DbNavdataAdapter navdata, RouteParserFactory routeParserFactory)
{
    protected DbNavdataAdapter Navdata => navdata;
    protected RouteParserFactory RouteParserFactory => routeParserFactory;

    public async Task<IList<FlightLeg>> ParseRouteAsync(string route, string dep, string arr, CancellationToken ct = default)
    {
        var parser = RouteParserFactory.Create($"{dep} {route} {arr}", Navdata);
        return await parser.Parse(ct);
    }

    public async Task<IList<ValidationMessage>> ValidateFlight(Flight flight, IList<FlightLeg> legs, CancellationToken ct = default)
    {
        var validator = new Validator(flight, legs, Navdata, RouteParserFactory);
        return await validator.Validate(ct);
    }
}
