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

    public async Task<Sheet> EnsureSheetAsync(string sheetId, string sheetName, CancellationToken ct = default)
    {
        var sheet = await GetSheetByIdAsync(sheetId, ct);
        if (sheet != null)
        {
            return sheet;
        }

        sheet = new Sheet
        {
            Id = sheetId,
            Name = sheetName,
            Fields = [],
        };

        dbContext.Sheet.Add(sheet);
        await dbContext.SaveChangesAsync(ct);

        return sheet;
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
                existingField.DescriptionZh = field.DescriptionZh;
                existingField.DescriptionEn = field.DescriptionEn;
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

    public async Task<SheetFiling> SetSheetFilingAsync(
        string sheetId,
        Ulid? sheetFilingId,
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

        SheetFiling sheetFiling;
        if (sheetFilingId == null)
        {
            sheetFiling = new SheetFiling
            {
                Id = Ulid.NewUlid(),
                SheetId = sheetId,
                UserId = userId,
                FiledAt = DateTime.UtcNow,
                Answers = [],
            };
            dbContext.SheetFiling.Add(sheetFiling);
        }
        else
        {
            sheetFiling = await GetSheetFilingByIdAsync(sheetFilingId.Value, ct)
                ?? throw new SheetFilingNotFoundException(sheetFilingId.Value, nameof(sheetFilingId));
        }

        var existingAnswers = sheetFiling.Answers.ToDictionary(a => a.FieldId, a => a);

        foreach (var field in sheet.Fields)
        {
            if (field.IsDeleted)
            {
                continue;
            }

            if (!answers.TryGetValue(field.Id, out string? value) || string.IsNullOrWhiteSpace(value))
            {
                throw new RequiredFieldMissingException(field.Id, sheetId, nameof(answers));
            }

            if (existingAnswers.TryGetValue(field.Id, out var existingAnswer))
            {
                existingAnswer.Answer = value;
            }
            else
            {
                var filingAnswer = new SheetFilingAnswer
                {
                    FilingId = sheetFiling.Id,
                    SheetId = sheetId,
                    FieldId = field.Id,
                    Answer = value,
                };
                sheetFiling.Answers.Add(filingAnswer);
            }
        }

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

    public class RequiredFieldMissingException(string fieldId, string sheetId, string paramName) :
        ArgumentException($"Required field {fieldId} is missing or empty for sheet {sheetId}", paramName)
    {
    }

    public class SheetFilingNotFoundException(Ulid filingId, string paramName) :
        ArgumentException($"Sheet filing {filingId} not found", paramName)
    {
    }
}
