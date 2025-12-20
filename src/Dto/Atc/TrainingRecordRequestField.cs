namespace Net.Vatprc.Uniapi.Dto;

public record TrainingRecordRequestField
{
    public required string Id { get; set; }
    public required string Answer { get; set; }
}
