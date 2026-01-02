namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationReviewRequest
{
    public required IEnumerable<SheetRequestField> ReviewAnswers { get; set; }
}
