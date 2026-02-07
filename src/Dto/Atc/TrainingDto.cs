using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingDto
{
    public required Ulid Id { get; init; }
    public required string Name { get; init; }
    public required Ulid TrainerId { get; init; }
    public required UserDto Trainer { get; init; }
    public required Ulid TraineeId { get; init; }
    public required UserDto Trainee { get; init; }
    public required DateTimeOffset StartAt { get; init; }
    public required DateTimeOffset EndAt { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }
    public DateTimeOffset? DeletedAt { get; init; }
    public required Ulid? RecordSheetFilingId { get; init; }
    public required IEnumerable<SheetFieldAnswerDto>? RecordSheetFiling { get; init; }

    public static TrainingDto From(Training training)
    {
        if (training.Trainer == null)
        {
            throw new ArgumentException("Training.Trainer is null", nameof(training));
        }

        if (training.Trainee == null)
        {
            throw new ArgumentException("Training.Trainee is null", nameof(training));
        }

        if (training.RecordSheetFilingId != null && training.RecordSheetFiling == null)
        {
            throw new ArgumentNullException(nameof(training), "RecordSheetFiling must be loaded");
        }

        if (training.RecordSheetFiling != null && training.RecordSheetFiling.Answers == null)
        {
            throw new ArgumentNullException(nameof(training), "RecordSheetFiling.Answers must be loaded");
        }

        return new TrainingDto
        {
            Id = training.Id,
            Name = training.Name,
            TrainerId = training.TrainerId,
            Trainer = UserDto.From(training.Trainer, true),
            TraineeId = training.TraineeId,
            Trainee = UserDto.From(training.Trainee, true),
            StartAt = training.StartAt,
            EndAt = training.EndAt,
            CreatedAt = training.CreatedAt,
            UpdatedAt = training.UpdatedAt,
            DeletedAt = training.DeletedAt,
            RecordSheetFilingId = training.RecordSheetFilingId,
            RecordSheetFiling = training.RecordSheetFiling?.Answers?.Select(SheetFieldAnswerDto.From),
        };
    }
}
