namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationReviewRequestAnswer
{
    public required string Id { get; set; }
    public required string Answer { get; set; }
}
