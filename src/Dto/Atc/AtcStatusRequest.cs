namespace Net.Vatprc.Uniapi.Dto;

public record AtcStatusRequest
{
    public required bool IsVisiting { get; init; }
    public required bool IsAbsent { get; init; }
    public required string Rating { get; init; }
    public required IEnumerable<AtcPermissionRequest> Permissions { get; init; }
}
