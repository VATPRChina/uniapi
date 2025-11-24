using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventBookingDto
{
    public Ulid Id { get; set; }
    public Ulid UserId { get; set; }
    public DateTimeOffset CreatedAt { get; set; }
    public DateTimeOffset UpdatedAt { get; set; }

    public EventBookingDto(EventBooking booking)
    {
        Id = booking.Id;
        UserId = booking.UserId;
        CreatedAt = booking.CreatedAt;
        UpdatedAt = booking.UpdatedAt;
    }
}
