using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationResponseDto
{
    public required Ulid Id { get; init; }
    public required Ulid ApplicationId { get; init; }
    public required Ulid TrainerId { get; init; }
    public required UserDto Trainer { get; init; }
    public required bool IsAccepted { get; init; }
    public required string Comment { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }

    public static TrainingApplicationResponseDto From(TrainingApplicationResponse resp)
    {
        if (resp.Application == null)
        {
            throw new ArgumentException("TrainingApplicationResponse.Application is null", nameof(resp));
        }

        if (resp.Trainer == null)
        {
            throw new ArgumentException("TrainingApplicationResponse.Trainer is null", nameof(resp));
        }

        return new TrainingApplicationResponseDto
        {
            Id = resp.Id,
            ApplicationId = resp.ApplicationId,
            TrainerId = resp.TrainerId,
            Trainer = UserDto.From(resp.Trainer, true),
            IsAccepted = resp.SlotId != null,
            Comment = resp.Comment,
            CreatedAt = resp.CreatedAt,
            UpdatedAt = resp.UpdatedAt
        };
    }
}
