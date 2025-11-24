using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventDto
{
    public Ulid Id { get; init; }
    public DateTimeOffset CreatedAt { get; init; }
    public DateTimeOffset UpdatedAt { get; init; }
    public string Title { get; init; }
    public DateTimeOffset StartAt { get; init; }
    public DateTimeOffset EndAt { get; init; }
    public DateTimeOffset StartBookingAt { get; init; }
    public DateTimeOffset EndBookingAt { get; init; }
    public string? ImageUrl { get; set; }
    public string Description { get; set; }

    public EventDto(Event eventt)
    {
        Id = eventt.Id;
        CreatedAt = eventt.CreatedAt;
        UpdatedAt = eventt.UpdatedAt;
        Title = eventt.Title;
        StartAt = eventt.StartAt;
        EndAt = eventt.EndAt;
        StartBookingAt = eventt.StartBookingAt;
        EndBookingAt = eventt.EndBookingAt;
        ImageUrl = eventt.ImageUrl;
        Description = eventt.Description;
    }
}
