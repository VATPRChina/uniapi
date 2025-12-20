using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingRecordFieldAnswerDto(
    SheetFieldDto Field,
    string Answer)
{
    public TrainingRecordFieldAnswerDto(SheetFilingAnswer answer) : this(
        new SheetFieldDto(answer.Field ??
            throw new ArgumentNullException(nameof(answer), "Field must be loaded")),
        answer.Answer)
    { }
}
