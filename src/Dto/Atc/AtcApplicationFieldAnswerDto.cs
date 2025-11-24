using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record AtcApplicationFieldAnswerDto(
    SheetFieldDto Field,
    string Answer)
{
    public AtcApplicationFieldAnswerDto(SheetFilingAnswer answer) : this(
        new SheetFieldDto(answer.Field ??
            throw new ArgumentNullException(nameof(answer), "Field must be loaded")),
        answer.Answer)
    { }
}
