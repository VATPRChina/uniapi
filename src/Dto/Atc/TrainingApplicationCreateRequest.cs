namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationCreateRequest
{
    public required string Name { get; init; }
    public required IEnumerable<TrainingApplicationCreateRequestSlot> Slots { get; init; }
}
