using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcPermissionRequest
{
    public required string PositionKindId { get; init; }
    public required UserControllerState State { get; init; }
    public DateTimeOffset? SoloExpiresAt { get; init; }
}
