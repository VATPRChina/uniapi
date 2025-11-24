using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationSheetDto(
    string Id,
    string Name,
    IEnumerable<AtcApplicationSheetFieldDto> Fields)
{
    public AtcApplicationSheetDto(Sheet sheet) : this(
        sheet.Id,
        sheet.Name,
        sheet.Fields
            .Where(field => !field.IsDeleted)
            .Select(field => new AtcApplicationSheetFieldDto(field)))
    { }
}
