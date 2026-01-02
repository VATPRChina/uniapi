using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationDto
{
    public required Ulid Id { get; init; }
    public required Ulid UserId { get; init; }
    public required UserDto User { get; init; }
    public required DateTimeOffset AppliedAt { get; init; }
    public required AtcApplicationStatus Status { get; init; }
    public required IEnumerable<SheetFieldAnswerDto> ApplicationFilingAnswers { get; init; }
    public IEnumerable<SheetFieldAnswerDto>? ReviewFilingAnswers { get; init; }

    public static AtcApplicationDto From(AtcApplication application, bool isAdmin, Ulid currentUserId)
    {
        if (application.User == null)
        {
            throw new ArgumentException("AtcApplication.User is null", nameof(application));
        }

        if (application.ApplicationFiling == null)
        {
            throw new ArgumentNullException(nameof(application), "ApplicationFiling must be loaded");
        }

        if (application.ApplicationFiling.Answers == null)
        {
            throw new ArgumentNullException(nameof(application), "ApplicationFiling.Answers must be loaded");
        }

        if (application.ReviewFilingId != null && application.ReviewFiling == null)
        {
            throw new ArgumentNullException(nameof(application), "ReviewFiling must be loaded");
        }

        if (application.ReviewFiling != null && application.ReviewFiling.Answers == null)
        {
            throw new ArgumentNullException(nameof(application), "ReviewFiling.Answers must be loaded");
        }

        return new()
        {
            Id = application.Id,
            UserId = application.UserId,
            User = UserDto.From(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded"),
            isAdmin || application.UserId == currentUserId),
            AppliedAt = application.AppliedAt,
            Status = application.Status,
            ApplicationFilingAnswers = application.ApplicationFiling.Answers.Select(SheetFieldAnswerDto.From),
            ReviewFilingAnswers = application.ReviewFiling?.Answers?.Select(SheetFieldAnswerDto.From),
        };
    }
}
