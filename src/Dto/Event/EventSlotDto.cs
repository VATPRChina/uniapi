using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventSlotDto
{
    public Ulid Id { get; set; }
    public Ulid EventId { get; set; }
    public Ulid AirspaceId { get; set; }
    public EventAirspaceDto Airspace { get; set; }
    public DateTimeOffset EnterAt { get; set; }
    public DateTimeOffset? LeaveAt { get; set; }
    public DateTimeOffset CreatedAt { get; set; }
    public DateTimeOffset UpdatedAt { get; set; }
    public EventBookingDto? Booking { get; set; }
    public string? Callsign { get; set; }
    public string? AircraftTypeIcao { get; set; }

    public EventSlotDto(EventSlot slot)
    {
        Id = slot.Id;
        EventId = slot.EventAirspace.EventId;
        AirspaceId = slot.EventAirspaceId;
        Airspace = new(slot.EventAirspace);
        EnterAt = slot.EnterAt;
        CreatedAt = slot.CreatedAt;
        UpdatedAt = slot.UpdatedAt;
        LeaveAt = slot.LeaveAt;
        if (slot.Booking != null) Booking = new(slot.Booking);
        Callsign = slot.Callsign;
        AircraftTypeIcao = slot.AircraftTypeIcao;
    }
}
