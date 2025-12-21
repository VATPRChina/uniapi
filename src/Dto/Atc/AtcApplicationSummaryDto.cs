using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationSummaryDto(
    Ulid Id,
    Ulid UserId,
    UserDto User,
    DateTimeOffset AppliedAt,
    AtcApplicationStatus Status)
{
    public AtcApplicationSummaryDto(AtcApplication application, bool isAdmin, Ulid currentUserId) : this(
        application.Id,
        application.UserId,
        new(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded"),
            isAdmin || application.UserId == currentUserId),
        application.AppliedAt,
        application.Status)
    { }
}
