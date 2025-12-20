namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationCreateRequest
{
    public required string Name { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
}
