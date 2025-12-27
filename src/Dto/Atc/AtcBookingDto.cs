using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcBookingDto
{
    public required Ulid Id { get; init; }
    public required UserDto User { get; init; }
    public required string Callsign { get; init; }
    public required DateTimeOffset BookedAt { get; init; }
    public required DateTimeOffset StartTime { get; init; }
    public required DateTimeOffset EndTime { get; init; }

    public static AtcBookingDto From(AtcBooking booking)
    {
        if (booking.User == null)
        {
            throw new ArgumentException("AtcBooking.User is null", nameof(booking));
        }

        return new AtcBookingDto
        {
            Id = booking.Id,
            User = UserDto.From(booking.User),
            Callsign = booking.Callsign,
            BookedAt = booking.BookedAt,
            StartTime = booking.StartAt,
            EndTime = booking.EndAt,
        };
    }
}
