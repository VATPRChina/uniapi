using Net.Vatprc.Uniapi.Models.Sheet;

namespace Net.Vatprc.Uniapi.Services;

public class SheetService(
    Database dbContext
)
{
    public async Task<Sheet?> GetSheetByIdAsync(string sheetId, CancellationToken ct = default)
    {
        return await dbContext.Sheet
            .Where(s => s.Id == sheetId)
            .Include(s => s.Fields.OrderBy(f => f.Sequence))
            .FirstOrDefaultAsync(ct);
    }

    public async Task<Sheet> SetSheetFieldsAsync(
        string sheetId,
        IEnumerable<SheetField> fields,
        CancellationToken ct = default)
    {
        var sheet = await GetSheetByIdAsync(sheetId, ct)
            ?? throw new ArgumentException("Sheet not found", nameof(sheetId));

        var existingFields = dbContext.SheetField
            .Where(f => f.SheetId == sheetId);

        dbContext.SheetField.RemoveRange(existingFields);

        foreach (var field in fields)
        {
            field.SheetId = sheetId;

            if (field.Kind == SheetFieldKind.SingleChoice && !field.SingleChoiceOptions.Any())
            {
                throw new ArgumentException($"Single choice field '{field.NameEn}' must have at least one option", nameof(fields));
            }
            else if (field.Kind != SheetFieldKind.SingleChoice)
            {
                field.SingleChoiceOptions = [];
            }

            dbContext.SheetField.Add(field);
        }

        await dbContext.SaveChangesAsync(ct);

        return sheet;
    }

    public async Task<SheetFiling?> GetSheetFilingByIdAsync(Ulid filingId, CancellationToken ct = default)
    {
        return await dbContext.SheetFiling
            .Where(f => f.Id == filingId)
            .Include(f => f.Answers!.OrderBy(a => a.FieldSequence))
            .FirstOrDefaultAsync(ct);
    }

    public async Task<SheetFiling> CreateSheetFilingAsync(
        string sheetId,
        Ulid userId,
        IDictionary<uint, string> answers,
        CancellationToken ct = default)
    {
        var sheet = await GetSheetByIdAsync(sheetId, ct)
            ?? throw new ArgumentException("Sheet not found", nameof(sheetId));

        foreach (var (fieldNumber, answerText) in answers)
        {
            var field = sheet.Fields.FirstOrDefault(f => f.Sequence == fieldNumber);
            if (field == null)
            {
                throw new ArgumentException($"Field number {fieldNumber} not found in sheet {sheetId}", nameof(answers));
            }
        }

        var sheetFiling = new SheetFiling
        {
            Id = Ulid.NewUlid(),
            SheetId = sheetId,
            UserId = userId,
            FiledAt = DateTime.UtcNow,
            Answers = [],
        };

        foreach (var field in sheet.Fields)
        {
            if (!answers.TryGetValue(field.Sequence, out string? value) || string.IsNullOrWhiteSpace(value))
            {
                throw new ArgumentException($"Required field number {field.Sequence} is missing or empty", nameof(answers));
            }

            var filingAnswer = new SheetFilingAnswer
            {
                FilingId = sheetFiling.Id,
                FieldSequence = field.Sequence,
                Answer = value,
            };

            sheetFiling.Answers.Add(filingAnswer);
        }

        dbContext.SheetFiling.Add(sheetFiling);

        await dbContext.SaveChangesAsync(ct);

        return sheetFiling;
    }
}
