using Net.Vatprc.Uniapi.Models.Event;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAtcPositionDto(
    string Callsign,
    DateTimeOffset StartAt,
    DateTimeOffset EndAt,
    string? Remarks,
    string PositionKindId,
    UserControllerState MinimumControllerState,
    EventAtcPositionBookingDto? Booking
)
{
    public EventAtcPositionDto(EventAtcPosition position) : this(
        position.Callsign,
        position.StartAt,
        position.EndAt,
        position.Remarks,
        position.PositionKindId,
        position.MinimumControllerState,
        position.Booking != null ? new EventAtcPositionBookingDto(position.Booking) : null)
    {
    }
}
