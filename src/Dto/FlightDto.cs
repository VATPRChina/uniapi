using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Dto;

public record FlightDto
{
    public required Ulid Id { get; init; }
    public required string Cid { get; init; }
    public required string Callsign { get; init; }
    public required DateTimeOffset LastObservedAt { get; init; }
    public required string Departure { get; init; }
    public required string Arrival { get; init; }
    public required string Equipment { get; init; }
    public required string NavigationPerformance { get; init; }
    public required string Transponder { get; init; }
    public required string RawRoute { get; init; }
    public required string Aircraft { get; init; }
    public required long Altitude { get; init; }
    public required long CruisingLevel { get; init; }

    public static FlightDto From(Flight flight)
    {
        return new()
        {
            Id = flight.Id,
            Cid = flight.Cid,
            Callsign = flight.Callsign,
            LastObservedAt = flight.LastObservedAt,
            Departure = flight.Departure,
            Arrival = flight.Arrival,
            Equipment = flight.Equipment,
            NavigationPerformance = flight.NavigationPerformance,
            Transponder = flight.Transponder,
            RawRoute = flight.RawRoute,
            Aircraft = flight.Aircraft,
            Altitude = flight.Altitude,
            CruisingLevel = flight.CruisingLevel,
        };
    }
}
