using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record SheetDto(
    string Id,
    string Name,
    IEnumerable<SheetFieldDto> Fields)
{
    public SheetDto(Sheet sheet) : this(
        sheet.Id,
        sheet.Name,
        sheet.Fields
            .Where(field => !field.IsDeleted)
            .Select(field => new SheetFieldDto(field)))
    { }
}
