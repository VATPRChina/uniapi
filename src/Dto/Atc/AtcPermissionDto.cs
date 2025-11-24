using Net.Vatprc.Uniapi.Models.Atc;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcPermissionDto(
    string PositionKindId,
    UserControllerState State,
    DateTimeOffset? SoloExpiresAt
)
{
    public AtcPermissionDto(UserAtcPermission permission) : this(
        permission.PositionKindId,
        permission.State,
        permission.SoloExpiresAt)
    {
    }
}
