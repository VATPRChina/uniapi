using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationDto(
    Ulid Id,
    Ulid UserId,
    UserDto User,
    DateTimeOffset AppliedAt,
    AtcApplicationStatus Status,
    IEnumerable<AtcApplicationFieldAnswerDto> ApplicationFilingAnswers,
    IEnumerable<AtcApplicationFieldAnswerDto>? ReviewFilingAnswers = null)
{
    public AtcApplicationDto(AtcApplication application) : this(
        application.Id,
        application.UserId,
        new(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded")),
        application.AppliedAt,
        application.Status,
        application.ApplicationFiling?.Answers.Select(answer => new AtcApplicationFieldAnswerDto(answer)) ??
            throw new ArgumentNullException(nameof(application), "ApplicationFiling must be loaded"),
        application.ReviewFiling?.Answers.Select(answer => new AtcApplicationFieldAnswerDto(answer)))
    { }
}
