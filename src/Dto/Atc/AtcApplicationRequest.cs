namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationRequest
{
    public required IEnumerable<AtcApplicationRequestField> ApplicationFilingAnswers { get; set; }
}
