using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Services.FlightPlan;

namespace Net.Vatprc.Uniapi.Adapters;

public class NavdataProvider(NavadataAdapter adapter) : INavdataProvider
{
    public Task<bool> ExistsAirwayWithFix(string ident, string fixIdent)
    {
        throw new NotImplementedException();
    }

    public Task<Airport?> FindAirport(string ident)
    {
        throw new NotImplementedException();
    }

    public Task<Fix?> FindFix(string ident, double lat, double lon)
    {
        throw new NotImplementedException();
    }

    public Task<Procedure?> FindSid(string ident, string airportIdent)
    {
        throw new NotImplementedException();
    }

    public Task<Procedure?> FindStar(string ident, string airportIdent)
    {
        throw new NotImplementedException();
    }

    public IAsyncEnumerable<AirwayLeg> GetAirwayLegs(string ident)
    {
        throw new NotImplementedException();
    }

    public Task<IList<PreferredRoute>> GetRecommendedRoutes(string dep, string arr)
    {
        return adapter.GetPreferredRouteAsync(dep, arr);
    }
}
