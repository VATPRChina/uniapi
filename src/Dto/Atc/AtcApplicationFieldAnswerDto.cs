using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationFieldAnswerDto(
    AtcApplicationSheetFieldDto Field,
    string Answer)
{
    public AtcApplicationFieldAnswerDto(SheetFilingAnswer answer) : this(
        new AtcApplicationSheetFieldDto(answer.Field ??
            throw new ArgumentNullException(nameof(answer), "Field must be loaded")),
        answer.Answer)
    { }
}
