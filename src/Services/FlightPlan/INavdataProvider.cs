using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Services.FlightPlan;

public interface INavdataProvider
{
    public Task<Airport?> FindAirport(string ident);
    public Task<bool> ExistsAirwayWithFix(string ident, string fixIdent);
    public Task<Procedure?> FindSid(string ident, string airportIdent);
    public Task<Procedure?> FindStar(string ident, string airportIdent);
    public Task<Fix?> FindFix(string ident, double lat, double lon);
    public IAsyncEnumerable<AirwayLeg> GetAirwayLegs(string ident);
    public Task<IList<PreferredRoute>> GetRecommendedRoutes(string dep, string arr);
}
