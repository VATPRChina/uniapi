using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationSummaryDto
{
    public required Ulid Id { get; init; }
    public required Ulid UserId { get; init; }
    public required UserDto User { get; init; }
    public required DateTimeOffset AppliedAt { get; init; }
    public required AtcApplicationStatus Status { get; init; }

    public static AtcApplicationSummaryDto From(AtcApplication application, bool isAdmin, Ulid currentUserId)
    {
        if (application.User == null)
        {
            throw new ArgumentNullException(nameof(application), "User must be loaded");
        }

        return new()
        {
            Id = application.Id,
            UserId = application.UserId,
            User = UserDto.From(application.User, isAdmin || application.UserId == currentUserId),
            AppliedAt = application.AppliedAt,
            Status = application.Status,
        };
    }
}
