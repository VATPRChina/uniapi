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
            .Include(s => s.Fields.OrderBy(f => !f.IsDeleted).ThenBy(f => f.Sequence))
            // .Where(f => !f.IsDeleted) does not work due to navigation fixup:
            // https://learn.microsoft.com/en-us/ef/core/querying/related-data/eager
            .FirstOrDefaultAsync(ct);
    }

    public async Task<Sheet> SetSheetFieldsAsync(
        string sheetId,
        IEnumerable<SheetField> fields,
        CancellationToken ct = default)
    {
        var sheet = await GetSheetByIdAsync(sheetId, ct)
            ?? new Sheet
            {
                Id = sheetId,
                Name = sheetId,
                Fields = [],
            };

        foreach (var field in fields)
        {
            field.SheetId = sheetId;

            if (field.Kind == SheetFieldKind.SingleChoice && !field.SingleChoiceOptions.Any())
            {
                throw new SingleChoiceOptionMissingException(field.Id, sheetId, nameof(fields));
            }
            else if (field.Kind != SheetFieldKind.SingleChoice)
            {
                field.SingleChoiceOptions = [];
            }

            if (!sheet.Fields.Any(f => f.Id == field.Id))
            {
                sheet.Fields.Add(field);
            }
            else
            {
                var existingField = sheet.Fields.First(f => f.Id == field.Id);
                existingField.Sequence = field.Sequence;
                existingField.NameZh = field.NameZh;
                existingField.NameEn = field.NameEn;
                existingField.Kind = field.Kind;
                existingField.SingleChoiceOptions = field.SingleChoiceOptions;
                existingField.IsDeleted = false;
            }
        }

        foreach (var existingField in sheet.Fields)
        {
            if (!fields.Any(f => f.Id == existingField.Id))
            {
                existingField.IsDeleted = true;
            }
        }

        await dbContext.SaveChangesAsync(ct);

        return sheet;
    }

    public async Task<SheetFiling?> GetSheetFilingByIdAsync(Ulid filingId, CancellationToken ct = default)
    {
        return await dbContext.SheetFiling
            .Where(f => f.Id == filingId)
            .Include(f => f.Answers!.OrderBy(a => a.Field!.Sequence))
            .FirstOrDefaultAsync(ct);
    }

    public async Task<SheetFiling> CreateSheetFilingAsync(
        string sheetId,
        Ulid userId,
        IDictionary<string, string> answers,
        CancellationToken ct = default)
    {
        var sheet = await GetSheetByIdAsync(sheetId, ct)
            ?? throw new SheetNotFoundException(nameof(sheetId));

        foreach (var (id, answerText) in answers)
        {
            var field = sheet.Fields.FirstOrDefault(f => f.Id == id);
            if (field == null)
            {
                throw new FieldNotFoundException(id, sheetId, nameof(answers));
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
            if (!answers.TryGetValue(field.Id, out string? value) || string.IsNullOrWhiteSpace(value))
            {
                throw new RequiredFieldMissingException(field.Sequence, sheetId, nameof(answers));
            }

            var filingAnswer = new SheetFilingAnswer
            {
                FilingId = sheetFiling.Id,
                FieldId = field.Id,
                Answer = value,
            };

            sheetFiling.Answers.Add(filingAnswer);
        }

        dbContext.SheetFiling.Add(sheetFiling);

        await dbContext.SaveChangesAsync(ct);

        return sheetFiling;
    }

    public class SheetNotFoundException(string paramName) :
        ArgumentException("Sheet not found", paramName)
    {
    }

    public class FieldNotFoundException(string fieldId, string sheetId, string paramName) :
        ArgumentException($"Field {fieldId} not found in sheet {sheetId}", paramName)
    {
    }

    public class SingleChoiceOptionMissingException(string fieldId, string sheetId, string paramName) :
        ArgumentException($"Single choice field {fieldId} must have at least one option in sheet {sheetId}", paramName)
    {
    }

    public class RequiredFieldMissingException(uint fieldSeq, string sheetId, string paramName) :
        ArgumentException($"Required field number {fieldSeq} is missing or empty for sheet {sheetId}", paramName)
    {
    }
}
