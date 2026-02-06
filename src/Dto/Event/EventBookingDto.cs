using System.Diagnostics.CodeAnalysis;
using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventBookingDto
{
    public required Ulid Id { get; set; }
    public required Ulid UserId { get; set; }
    public UserDto? User { get; set; }
    public required DateTimeOffset CreatedAt { get; set; }
    public required DateTimeOffset UpdatedAt { get; set; }

    [return: NotNullIfNotNull(nameof(booking))]
    public static EventBookingDto? From(EventBooking? booking, bool includeUser = false)
    {
        if (booking == null) return null;
        return new()
        {
            Id = booking.Id,
            UserId = booking.UserId,
            User = includeUser && booking.User != null ? UserDto.From(booking.User, false) : null,
            CreatedAt = booking.CreatedAt,
            UpdatedAt = booking.UpdatedAt,
        };
    }
}
