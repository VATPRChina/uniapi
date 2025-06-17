using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.External.FlightPlan;

public interface INavdataProvider
{
    public Task<Airport?> FindAirport(string ident);
    public Task<bool> ExistsAirwayWithFix(string ident, string fixIdent);
    public Task<Procedure?> FindSid(string ident, string airportIdent);
    public Task<Procedure?> FindStar(string ident, string airportIdent);
    public Task<VhfNavaid?> FindVhfNavaid(string ident, double lat, double lon);
    public Task<NdbNavaid?> FindNdbNavaid(string ident, double lat, double lon);
    public Task<Waypoint?> FindWaypoint(string ident, double lat, double lon);
}
