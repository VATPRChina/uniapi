using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record SheetFieldDto
{
    public required string SheetId { get; init; }
    public required string Id { get; init; }
    public required uint Sequence { get; init; }
    public required string NameZh { get; init; }
    public string? NameEn { get; init; }
    public required SheetFieldKind Kind { get; init; }
    public required IEnumerable<string> SingleChoiceOptions { get; init; }
    public string? DescriptionZh { get; init; }
    public string? DescriptionEn { get; init; }
    public required bool IsDeleted { get; init; }

    public static SheetFieldDto From(SheetField field)
    {
        return new()
        {
            SheetId = field.SheetId,
            Id = field.Id,
            Sequence = field.Sequence,
            NameZh = field.NameZh,
            NameEn = field.NameEn,
            Kind = field.Kind,
            SingleChoiceOptions = field.SingleChoiceOptions,
            DescriptionZh = field.DescriptionZh,
            DescriptionEn = field.DescriptionEn,
            IsDeleted = field.IsDeleted,
        };
    }
}
