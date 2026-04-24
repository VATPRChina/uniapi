using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record SheetFieldSaveRequest
{
    public required string Id { get; init; }
    public required uint Sequence { get; init; }
    public required string NameZh { get; init; }
    public string? NameEn { get; init; }
    public required SheetFieldKind Kind { get; init; }
    public IEnumerable<string>? SingleChoiceOptions { get; init; }
    public string? DescriptionZh { get; init; }
    public string? DescriptionEn { get; init; }
}
