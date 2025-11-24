using Net.Vatprc.Uniapi.Models.Atc;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcPermissionSetRequest
{
    public required UserAtcPermission.UserControllerState State { get; set; }
    public DateTimeOffset? SoloExpiresAt { get; set; }
}
