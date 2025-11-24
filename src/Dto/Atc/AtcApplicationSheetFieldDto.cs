using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationSheetFieldDto(
    string SheetId,
    string Id,
    uint Sequence,
    string NameZh,
    string? NameEn,
    SheetFieldKind Kind,
    IEnumerable<string> SingleChoiceOptions,
    bool IsDeleted)
{
    public AtcApplicationSheetFieldDto(SheetField field) : this(
        field.SheetId,
        field.Id,
        field.Sequence,
        field.NameZh,
        field.NameEn,
        field.Kind,
        field.SingleChoiceOptions,
        field.IsDeleted)
    { }
}
