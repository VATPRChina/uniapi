using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationReviewRequest
{
    public required AtcApplicationStatus Status { get; set; }
    public required IEnumerable<SheetRequestField> ReviewAnswers { get; set; }
}
