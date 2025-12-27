using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventSlotDto
{
    public required Ulid Id { get; set; }
    public required Ulid EventId { get; set; }
    public required Ulid AirspaceId { get; set; }
    public required EventAirspaceDto Airspace { get; set; }
    public required DateTimeOffset EnterAt { get; set; }
    public required DateTimeOffset? LeaveAt { get; set; }
    public required DateTimeOffset CreatedAt { get; set; }
    public required DateTimeOffset UpdatedAt { get; set; }
    public required EventBookingDto? Booking { get; set; }
    public required string? Callsign { get; set; }
    public required string? AircraftTypeIcao { get; set; }

    public static EventSlotDto From(EventSlot slot)
    {
        return new()
        {
            Id = slot.Id,
            EventId = slot.EventAirspace.EventId,
            AirspaceId = slot.EventAirspaceId,
            Airspace = EventAirspaceDto.From(slot.EventAirspace),
            EnterAt = slot.EnterAt,
            CreatedAt = slot.CreatedAt,
            UpdatedAt = slot.UpdatedAt,
            LeaveAt = slot.LeaveAt,
            Booking = EventBookingDto.From(slot.Booking),
            Callsign = slot.Callsign,
            AircraftTypeIcao = slot.AircraftTypeIcao,
        };
    }
}
