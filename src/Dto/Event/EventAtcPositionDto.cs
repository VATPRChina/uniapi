using Net.Vatprc.Uniapi.Models.Event;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAtcPositionDto
{
    public required Ulid Id { get; init; }
    public required EventDto Event { get; init; }
    public required string Callsign { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
    public string? Remarks { get; init; }
    public required string PositionKindId { get; init; }
    public required UserControllerState MinimumControllerState { get; init; }
    public EventAtcPositionBookingDto? Booking { get; init; }

    public static EventAtcPositionDto From(EventAtcPosition position)
    {
        if (position.Event == null) throw new ArgumentNullException(nameof(position), "Event must be loaded");

        return new()
        {
            Id = position.Id,
            Event = EventDto.From(position.Event),
            Callsign = position.Callsign,
            StartAt = position.StartAt,
            EndAt = position.EndAt,
            Remarks = position.Remarks,
            PositionKindId = position.PositionKindId,
            MinimumControllerState = position.MinimumControllerState,
            Booking = EventAtcPositionBookingDto.From(position.Booking),
        };
    }
}
