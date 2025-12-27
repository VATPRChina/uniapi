using Net.Vatprc.Uniapi.Models.Atc;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcPermissionDto
{
    public required string PositionKindId { get; init; }
    public required UserControllerState State { get; init; }
    public DateTimeOffset? SoloExpiresAt { get; init; }

    public static AtcPermissionDto From(UserAtcPermission permission)
    {
        return new()
        {
            PositionKindId = permission.PositionKindId,
            State = permission.State,
            SoloExpiresAt = permission.SoloExpiresAt,
        };
    }
}
