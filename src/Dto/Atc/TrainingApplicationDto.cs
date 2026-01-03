using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationDto
{
    public required Ulid Id { get; init; }
    public required Ulid TraineeId { get; init; }
    public required UserDto Trainee { get; init; }
    public required TrainingApplicationStatus Status { get; init; }
    public required string Name { get; init; }
    public Ulid? TrainId { get; init; }
    public IEnumerable<TrainingApplicationSlotDto>? Slots { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }

    public static TrainingApplicationDto From(TrainingApplication app)
    {
        ArgumentNullException.ThrowIfNull(app.Trainee, nameof(app.Trainee));
        ArgumentNullException.ThrowIfNull(app.Slots, nameof(app.Slots));

        TrainingApplicationStatus status;
        if (app.TrainId != null)
        {
            status = TrainingApplicationStatus.Accepted;
        }
        else
        {
            var endAt = app.Slots.Max(s => s.EndAt);
            if (endAt < DateTimeOffset.UtcNow)
            {
                status = TrainingApplicationStatus.Rejected;
            }
            else
            {
                status = TrainingApplicationStatus.Pending;
            }
        }

        return new TrainingApplicationDto
        {
            Id = app.Id,
            TraineeId = app.TraineeId,
            Trainee = UserDto.From(app.Trainee, true),
            Status = status,
            Name = app.Name,
            TrainId = app.TrainId,
            Slots = app.Slots.Select(TrainingApplicationSlotDto.From),
            CreatedAt = app.CreatedAt,
            UpdatedAt = app.UpdatedAt,
        };
    }
}
