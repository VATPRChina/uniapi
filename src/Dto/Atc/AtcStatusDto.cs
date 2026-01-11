using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record class AtcStatusDto
{
    public required Ulid UserId { get; init; }
    public required UserDto User { get; init; }
    public required bool IsVisiting { get; init; }
    public required bool IsAbsent { get; init; }
    public required string Rating { get; init; }
    public required IEnumerable<AtcPermissionDto> Permissions { get; init; }

    public static AtcStatusDto From(User user, UserAtcStatus? status, IEnumerable<UserAtcPermission> permissions)
    {
        return new AtcStatusDto
        {
            UserId = status?.UserId ?? user.Id,
            User = UserDto.From(user, showFullName: true),
            IsVisiting = status?.IsVisiting ?? false,
            IsAbsent = status?.IsAbsent ?? false,
            Rating = status?.Rating ?? "OBS",
            Permissions = permissions.Select(AtcPermissionDto.From),
        };
    }
}
