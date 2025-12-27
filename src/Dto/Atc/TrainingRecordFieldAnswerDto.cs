using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Dto;

public record TrainingRecordFieldAnswerDto
{
    public required SheetFieldDto Field { get; init; }
    public required string Answer { get; init; }

    public static TrainingRecordFieldAnswerDto From(SheetFilingAnswer answer)
    {
        return new()
        {
            Field = SheetFieldDto.From(answer.Field ??
            throw new ArgumentNullException(nameof(answer), "Field must be loaded")),
            Answer = answer.Answer,
        };
    }
}
