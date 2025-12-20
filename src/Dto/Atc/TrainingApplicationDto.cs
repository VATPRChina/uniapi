using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingApplicationDto
{
    public required Ulid Id { get; init; }
    public required Ulid TraineeId { get; init; }
    public required UserDto Trainee { get; init; }
    public required string Name { get; init; }
    public Ulid? TrainId { get; init; }
    public TrainingDto? Train { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }

    public static TrainingApplicationDto From(TrainingApplication app)
    {
        if (app.Trainee == null)
        {
            throw new ArgumentException("TrainingApplication.Trainee is null", nameof(app));
        }

        return new TrainingApplicationDto
        {
            Id = app.Id,
            TraineeId = app.TraineeId,
            Trainee = new UserDto(app.Trainee, true),
            Name = app.Name,
            TrainId = app.TrainId,
            Train = app.Train != null ? TrainingDto.From(app.Train) : null,
            StartAt = app.StartAt,
            EndAt = app.EndAt,
            CreatedAt = app.CreatedAt,
            UpdatedAt = app.UpdatedAt
        };
    }
}
