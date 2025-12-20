namespace Net.Vatprc.Uniapi.Dto;

public record TrainingRecordRequest
{
    public required IEnumerable<TrainingRecordRequestField> RequestAnswers { get; set; }
}
