namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationResponseRequest
{
    public required bool IsAccepted { get; init; }
    public required string Comment { get; init; }
}
