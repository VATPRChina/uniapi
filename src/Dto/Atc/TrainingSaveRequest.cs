using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingSaveRequest
{
    public required string Name { get; init; }
    public required Ulid TrainerId { get; init; }
    public required Ulid TraineeId { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
}
