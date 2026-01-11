namespace Net.Vatprc.Uniapi.Dto;

public record EventSaveRequest
{
    public required string Title { get; set; }
    public required DateTimeOffset StartAt { get; set; }
    public required DateTimeOffset EndAt { get; set; }
    public DateTimeOffset? StartBookingAt { get; init; }
    public DateTimeOffset? EndBookingAt { get; init; }
    public DateTimeOffset? StartAtcBookingAt { get; init; }
    public string? ImageUrl { get; init; }
    public required string Description { get; set; }
}
