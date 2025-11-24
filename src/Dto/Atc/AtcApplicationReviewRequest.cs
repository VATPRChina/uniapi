namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationReviewRequest
{
    public required IEnumerable<AtcApplicationReviewRequestAnswer> Answers { get; set; }
}
