using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Dto;

public record FlightDto(
    Ulid Id,
    string Cid,
    string Callsign,
    DateTimeOffset LastObservedAt,
    string Departure,
    string Arrival,
    string Equipment,
    string NavigationPerformance,
    string Transponder,
    string RawRoute,
    string Aircraft,
    long Altitude,
    long CruisingLevel)
{
    public FlightDto(Flight flight) : this(
        flight.Id,
        flight.Cid,
        flight.Callsign,
        flight.LastObservedAt,
        flight.Departure,
        flight.Arrival,
        flight.Equipment,
        flight.NavigationPerformance,
        flight.Transponder,
        flight.RawRoute,
        flight.Aircraft,
        flight.Altitude,
        flight.CruisingLevel)
    { }
}
