using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationDto
{
    public required Ulid Id { get; init; }
    public required Ulid UserId { get; init; }
    public required UserDto User { get; init; }
    public required DateTimeOffset AppliedAt { get; init; }
    public required AtcApplicationStatus Status { get; init; }
    public required IEnumerable<AtcApplicationFieldAnswerDto> ApplicationFilingAnswers { get; init; }
    public IEnumerable<AtcApplicationFieldAnswerDto>? ReviewFilingAnswers { get; init; }

    public static AtcApplicationDto From(AtcApplication application, bool isAdmin, Ulid currentUserId)
    {
        return new()
        {
            Id = application.Id,
            UserId = application.UserId,
            User = UserDto.From(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded"),
            isAdmin || application.UserId == currentUserId),
            AppliedAt = application.AppliedAt,
            Status = application.Status,
            ApplicationFilingAnswers = application.ApplicationFiling?.Answers.Select(AtcApplicationFieldAnswerDto.From) ??
                throw new ArgumentNullException(nameof(application), "ApplicationFiling must be loaded"),
            ReviewFilingAnswers = application.ReviewFiling?.Answers.Select(AtcApplicationFieldAnswerDto.From),
        };
    }
}
