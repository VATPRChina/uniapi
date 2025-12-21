using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record SheetFieldDto(
    string SheetId,
    string Id,
    uint Sequence,
    string NameZh,
    string? NameEn,
    SheetFieldKind Kind,
    IEnumerable<string> SingleChoiceOptions,
    string? DescriptionZh,
    string? DescriptionEn,
    bool IsDeleted)
{
    public SheetFieldDto(SheetField field) : this(
        field.SheetId,
        field.Id,
        field.Sequence,
        field.NameZh,
        field.NameEn,
        field.Kind,
        field.SingleChoiceOptions,
        field.DescriptionZh,
        field.DescriptionEn,
        field.IsDeleted)
    { }
}
