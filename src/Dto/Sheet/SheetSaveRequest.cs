namespace Net.Vatprc.Uniapi.Dto;

public record SheetSaveRequest
{
    public required string Name { get; init; }
    public required IEnumerable<SheetFieldSaveRequest> Fields { get; init; }
}
