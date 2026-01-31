using Arinc424.Ground;
using Arinc424.Navigation;
using Arinc424.Procedures;
using Arinc424.Waypoints;
using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

namespace Net.Vatprc.Uniapi.Services.FlightPlan;

public interface INavdataProvider
{
    public Airport? FindAirport(string ident);
    public bool ExistsAirwayWithFix(string ident, string fixIdent);
    public Departure? FindSid(string ident, string airportIdent);
    public Arrival? FindStar(string ident, string airportIdent);
    public Omnidirect? FindVhfNavaid(string ident, double lat, double lon);
    public Nondirect? FindNdbNavaid(string ident, double lat, double lon);
    public Waypoint? FindWaypoint(string ident, double lat, double lon);
    public IEnumerable<AirwayLeg> FindAirwayLegs(string ident);
}
