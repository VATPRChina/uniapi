using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventDto
{
    public required Ulid Id { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }
    public required string Title { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
    public required DateTimeOffset StartBookingAt { get; init; }
    public required DateTimeOffset EndBookingAt { get; init; }
    public required string? ImageUrl { get; set; }
    public required string Description { get; set; }

    public static EventDto From(Event eventt)
    {
        return new()
        {
            Id = eventt.Id,
            CreatedAt = eventt.CreatedAt,
            UpdatedAt = eventt.UpdatedAt,
            Title = eventt.Title,
            StartAt = eventt.StartAt,
            EndAt = eventt.EndAt,
            StartBookingAt = eventt.StartBookingAt,
            EndBookingAt = eventt.EndBookingAt,
            ImageUrl = eventt.ImageUrl,
            Description = eventt.Description,
        };
    }
}
