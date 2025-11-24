namespace Net.Vatprc.Uniapi.Dto;

public record EventSaveRequest
{
    public required string Title { get; set; }
    public required DateTimeOffset StartAt { get; set; }
    public required DateTimeOffset EndAt { get; set; }
    public required DateTimeOffset StartBookingAt { get; init; }
    public required DateTimeOffset EndBookingAt { get; init; }
    public string? ImageUrl { get; init; }
    public required string Description { get; set; }
}
