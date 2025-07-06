using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
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
    public IAsyncEnumerable<AirwayLeg> FindAirwayLegs(string ident);
    // TODO: public Task<FlightFix?> FindLastFixOfSid(string ident, string icao);
    // TODO: public Task<FlightFix?> FindLastFixOfStar(string ident, string icao);

    public record AirwayLeg
    {
        public required string Ident { get; init; }
        public required string FromFixIcaoCode { get; init; }
        public required string FromFixIdentifier { get; init; }
        public required Ulid FromFixId { get; init; }
        public required FixType FromFixType { get; init; }
        public required string ToFixIcaoCode { get; init; }
        public required string ToFixIdentifier { get; init; }
        public required Ulid ToFixId { get; init; }
        public required FixType ToFixType { get; init; }
    }

    public enum FixType
    {
        Waypoint,
        Vhf,
        Ndb,
        Unknown
    }
}
