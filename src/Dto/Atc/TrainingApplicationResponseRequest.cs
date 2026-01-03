namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationResponseRequest
{
    public Ulid? SlotId { get; set; }
    public required string Comment { get; init; }
}
