namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationRequest
{
    public required IEnumerable<SheetRequestField> RequestAnswers { get; set; }
}
