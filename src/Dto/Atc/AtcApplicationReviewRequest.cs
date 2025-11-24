namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationReviewRequest
{
    public required IEnumerable<AtcApplicationReviewRequestAnswer> ReviewAnswers { get; set; }
}
