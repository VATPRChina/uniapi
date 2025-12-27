using System.Diagnostics.CodeAnalysis;
using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAtcPositionBookingDto
{
    public required Ulid UserId { get; init; }
    public required DateTimeOffset BookedAt { get; init; }

    [return: NotNullIfNotNull(nameof(booking))]
    public static EventAtcPositionBookingDto? From(EventAtcPositionBooking? booking)
    {
        if (booking == null) return null;
        return new()
        {
            UserId = booking.UserId,
            BookedAt = booking.CreatedAt,
        };
    }
}
