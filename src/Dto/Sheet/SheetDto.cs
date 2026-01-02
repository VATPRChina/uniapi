using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record SheetDto
{
    public required string Id { get; init; }
    public required string Name { get; init; }
    public required IEnumerable<SheetFieldDto> Fields { get; init; }

    public static SheetDto From(Sheet sheet)
    {
        if (sheet.Fields == null)
        {
            throw new ArgumentNullException(nameof(sheet), "Fields must be loaded");
        }

        return new()
        {
            Id = sheet.Id,
            Name = sheet.Name,
            Fields = sheet.Fields
                .Where(field => !field.IsDeleted)
                .Select(SheetFieldDto.From),
        };
    }
}
