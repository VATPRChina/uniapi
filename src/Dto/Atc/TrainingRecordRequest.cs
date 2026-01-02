namespace Net.Vatprc.Uniapi.Dto;

public record TrainingRecordRequest
{
    public required IEnumerable<SheetRequestField> RequestAnswers { get; set; }
}
