namespace Net.Vatprc.Uniapi.Dto;

public record class ControllerDto
{
    public required UserDto User { get; init; }
    public required IEnumerable<AtcPermissionDto> Permissions { get; init; }
}
