using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAtcPositionBookingDto(
    Ulid UserId,
    DateTimeOffset BookedAt
)
{
    public EventAtcPositionBookingDto(EventAtcPositionBooking booking) : this(
        booking.UserId,
        booking.CreatedAt)
    {
    }
}
