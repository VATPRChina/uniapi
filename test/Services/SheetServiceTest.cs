using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using static Net.Vatprc.Uniapi.Services.SheetService;

namespace Net.Vatprc.Uniapi.Test.Services;

[TestFixture]
public class SheetServiceTest : TestWithDatabase
{
    private SheetService sheetService;
    private User? user;

    [SetUp]
    public void Setup()
    {
        sheetService = new SheetService(dbContext);
    }

    [Test]
    public async Task GetSheetByIdAsync_SheetExists_ReturnsSheet()
    {
        SetupSheet();

        var result = await sheetService.GetSheetByIdAsync("test-sheet");

        result.Should().NotBeNull();
        result.Id.Should().Be("test-sheet");
        result.Fields.Should().HaveCount(4);
    }

    [Test]
    public async Task GetSheetByIdAsync_SheetNotExists_ReturnsNull()
    {
        SetupSheet();

        var result = await sheetService.GetSheetByIdAsync("test-sheet-b");

        result.Should().BeNull();
    }

    [Test]
    public async Task EnsureSheetAsync_SheetExists_ReturnsExistingSheet()
    {
        SetupSheet();

        var result = await sheetService.EnsureSheetAsync("test-sheet", "Should Not Used");

        result.Should().NotBeNull();
        result.Id.Should().Be("test-sheet");
        result.Name.Should().Be("Test Sheet");
        result.Fields.Should().HaveCount(4);
    }

    [Test]
    public async Task EnsureSheetAsync_SheetNotExists_CreatesNewSheet()
    {
        var result = await sheetService.EnsureSheetAsync("created-sheet", "Created Sheet");

        result.Should().NotBeNull();
        result.Id.Should().Be("created-sheet");
        result.Name.Should().Be("Created Sheet");
        result.Fields.Should().BeEmpty();

        var persisted = dbContext.Sheet.Find(result.Id);
        persisted.Should().NotBeNull();
        persisted!.Name.Should().Be("Created Sheet");
    }

    [Test]
    public async Task SetSheetFieldsAsync_UpdatesField()
    {
        SetupSheet();

        var updatedFields = new List<SheetField>
        {
            new SheetField
            {
                SheetId = "test-sheet",
                Id = "field-one",
                Sequence = 1,
                NameZh = "字段一更新",
                NameEn = "Field One Updated",
                Kind = SheetFieldKind.ShortText,
            },
        };
        var result = await sheetService.SetSheetFieldsAsync("test-sheet", updatedFields);

        result.Fields.Should().HaveCount(4);
        var updatedField = result.Fields.First(f => f.Id == "field-one");
        updatedField.NameZh.Should().Be("字段一更新");
        updatedField.NameEn.Should().Be("Field One Updated");
    }

    [Test]
    public async Task SetSheetFieldsAsync_AddsNewField()
    {
        SetupSheet();

        var updatedFields = new List<SheetField>
        {
            new SheetField
            {
                SheetId = "test-sheet",
                Id = "field-five",
                Sequence = 5,
                NameZh = "字段五",
                NameEn = "Field Five",
                Kind = SheetFieldKind.LongText,
            },
        };
        var result = await sheetService.SetSheetFieldsAsync("test-sheet", updatedFields);

        result.Fields.Should().HaveCount(5);
        var newField = result.Fields.First(f => f.Id == "field-five");
        newField.NameZh.Should().Be("字段五");
        newField.NameEn.Should().Be("Field Five");
    }

    [Test]
    public async Task SetSheetFieldsAsync_MarksMissingFieldAsDeleted()
    {
        SetupSheet();

        var updatedFields = new List<SheetField>
        {
            new SheetField
            {
                SheetId = "test-sheet",
                Id = "field-one",
                Sequence = 1,
                NameZh = "字段一",
                NameEn = "Field One",
                Kind = SheetFieldKind.ShortText,
            },
        };
        var result = await sheetService.SetSheetFieldsAsync("test-sheet", updatedFields);

        result.Fields.Should().HaveCount(4);
        result.Fields.First(f => f.Id == "field-one")
            .IsDeleted.Should().BeFalse();
        result.Fields.First(f => f.Id == "field-two")
            .IsDeleted.Should().BeTrue();
        result.Fields.First(f => f.Id == "field-three")
            .IsDeleted.Should().BeTrue();
        result.Fields.First(f => f.Id == "field-four")
            .IsDeleted.Should().BeTrue();
    }

    [Test]
    public async Task SetSheetFieldsAsync_SingleChoiceFieldWithoutOptions_ThrowsArgumentException()
    {
        SetupSheet();

        var updatedFields = new List<SheetField>
        {
            new SheetField
            {
                SheetId = "test-sheet",
                Id = "field-six",
                Sequence = 6,
                NameZh = "字段六",
                NameEn = "Field Six",
                Kind = SheetFieldKind.SingleChoice,
            },
        };

        Func<Task> act = async () => await sheetService.SetSheetFieldsAsync("test-sheet", updatedFields);

        await act.Should().ThrowAsync<SingleChoiceOptionMissingException>()
            .WithMessage("Single choice field field-six must have at least one option in sheet test-sheet (Parameter 'fields')")
            .Where(e => e.ParamName == "fields");
    }

    [Test]
    public async Task SetSheetFieldsAsync_SheetNotFound_CreatesNewSheet()
    {
        var newFields = new List<SheetField>
        {
            new SheetField
            {
                SheetId = "new-sheet",
                Id = "field-one",
                Sequence = 1,
                NameZh = "字段一",
                NameEn = "Field One",
                Kind = SheetFieldKind.ShortText,
            },
        };
        var result = await sheetService.SetSheetFieldsAsync("new-sheet", newFields);

        result.Id.Should().Be("new-sheet");
        result.Name.Should().Be("new-sheet");
        result.Fields.Should().HaveCount(1);
        var newField = result.Fields.First(f => f.Id == "field-one");
        newField.NameZh.Should().Be("字段一");
        newField.NameEn.Should().Be("Field One");
    }

    [Test]
    public async Task GetSheetFilingByIdAsync_Returns()
    {
        SetupSheet();
        SetupUser();
        SetupSheetFiling();

        var existingFiling = dbContext.SheetFiling.First();
        var result = await sheetService.GetSheetFilingByIdAsync(existingFiling.Id);

        result.Should().NotBeNull();
        result!.Id.Should().Be(existingFiling.Id);
    }

    [Test]
    public async Task CreateSheetFilingAsync_CreatesFiling()
    {
        SetupSheet();
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            { "field-two", "Test Answer 2" },
            { "field-three", "Option A" },
            {"field-four", "Should be ignored"}
        };
        var result = await sheetService.SetSheetFilingAsync("test-sheet", null, user!.Id, answers);

        result.Should().NotBeNull();
        result.SheetId.Should().Be("test-sheet");
        result.UserId.Should().Be(user.Id);
        result.Answers.Should().HaveCount(3);
        result.Answers.ElementAt(0).FieldId.Should().Be("field-one");
        result.Answers.ElementAt(0).Answer.Should().Be("Test Answer");
        result.Answers.ElementAt(1).FieldId.Should().Be("field-two");
        result.Answers.ElementAt(1).Answer.Should().Be("Test Answer 2");
        result.Answers.ElementAt(2).FieldId.Should().Be("field-three");
        result.Answers.ElementAt(2).Answer.Should().Be("Option A");
    }

    [Test]
    public async Task CreateSheetFilingAsync_CreatesFiling_AllowsEmptyDeletedField()
    {
        SetupSheet();
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            { "field-two", "Test Answer 2" },
            { "field-three", "Option A" },
        };
        var result = await sheetService.SetSheetFilingAsync("test-sheet", null, user!.Id, answers);

        result.Should().NotBeNull();
        result.SheetId.Should().Be("test-sheet");
        result.UserId.Should().Be(user.Id);
        result.Answers.Should().HaveCount(3);
        result.Answers.ElementAt(0).FieldId.Should().Be("field-one");
        result.Answers.ElementAt(0).Answer.Should().Be("Test Answer");
        result.Answers.ElementAt(1).FieldId.Should().Be("field-two");
        result.Answers.ElementAt(1).Answer.Should().Be("Test Answer 2");
        result.Answers.ElementAt(2).FieldId.Should().Be("field-three");
        result.Answers.ElementAt(2).Answer.Should().Be("Option A");
    }

    [Test]
    public async Task CreateSheetFilingAsync_MissingRequiredField_ThrowsException()
    {
        SetupSheet();
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            // Missing field-two
            { "field-three", "Option A" },
        };
        Func<Task> act = async () => await sheetService.SetSheetFilingAsync("test-sheet", null, user!.Id, answers);

        await act.Should().ThrowAsync<RequiredFieldMissingException>()
            .WithMessage("Required field field-two is missing or empty for sheet test-sheet (Parameter 'answers')")
            .Where(e => e.ParamName == "answers");
    }

    [Test]
    public async Task CreateSheetFilingAsync_EmptyRequiredField_ThrowsException()
    {
        SetupSheet();
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            { "field-two", "   " }, // Empty answer
            { "field-three", "Option A" },
        };
        Func<Task> act = async () => await sheetService.SetSheetFilingAsync("test-sheet", null, user!.Id, answers);

        await act.Should().ThrowAsync<RequiredFieldMissingException>()
            .WithMessage("Required field field-two is missing or empty for sheet test-sheet (Parameter 'answers')")
            .Where(e => e.ParamName == "answers");
    }

    [Test]
    public async Task CreateSheetFilingAsync_AnswersNonExistentField_ThrowsException()
    {
        SetupSheet();
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            { "field-two", "Test Answer 2" },
            { "field-three", "Option A" },
            { "non-existent-field", "Some Answer" },
        };
        Func<Task> act = async () => await sheetService.SetSheetFilingAsync("test-sheet", null, user!.Id, answers);

        await act.Should().ThrowAsync<FieldNotFoundException>()
            .WithMessage("Field non-existent-field not found in sheet test-sheet (Parameter 'answers')")
            .Where(e => e.ParamName == "answers");
    }

    [Test]
    public async Task CreateSheetFilingAsync_SheetNotFound_ThrowsException()
    {
        SetupUser();

        var answers = new Dictionary<string, string>
        {
            { "field-one", "Test Answer" },
            { "field-two", "Test Answer 2" },
            { "field-three", "Option A" },
        };
        Func<Task> act = async () => await sheetService.SetSheetFilingAsync("non-existent-sheet", null, user!.Id, answers);

        await act.Should().ThrowAsync<SheetNotFoundException>()
            .WithMessage("Sheet not found (Parameter 'sheetId')")
            .Where(e => e.ParamName == "sheetId");
    }

    [Test]
    public async Task SetSheetFilingAsync_UpdateExistingFiling_UpdatesAnswers()
    {
        SetupSheet();
        SetupUser();
        var filingId = SetupSheetFiling();

        // Initial answers
        var initialAnswers = new Dictionary<string, string>
        {
            { "field-one", "Initial Answer 1" },
            { "field-two", "Initial Answer 2" },
            { "field-three", "Option B" },
        };
        var filing1 = await sheetService.SetSheetFilingAsync("test-sheet", filingId, user!.Id, initialAnswers);
        filing1.Answers.Should().HaveCount(3);
        filing1.Answers.ElementAt(0).FieldId.Should().Be("field-one");
        filing1.Answers.ElementAt(0).Answer.Should().Be("Initial Answer 1");
        filing1.Answers.ElementAt(1).FieldId.Should().Be("field-two");
        filing1.Answers.ElementAt(1).Answer.Should().Be("Initial Answer 2");
        filing1.Answers.ElementAt(2).FieldId.Should().Be("field-three");
        filing1.Answers.ElementAt(2).Answer.Should().Be("Option B");

        // Updated answers
        var updatedAnswers = new Dictionary<string, string>
        {
            { "field-one", "Updated Answer 1" },
            { "field-two", "Updated Answer 2" },
            { "field-three", "Option C" },
        };
        var filing2 = await sheetService.SetSheetFilingAsync("test-sheet", filingId, user!.Id, updatedAnswers);

        filing2.Answers.Should().HaveCount(3);
        filing2.Answers.ElementAt(0).FieldId.Should().Be("field-one");
        filing2.Answers.ElementAt(0).Answer.Should().Be("Updated Answer 1");
        filing2.Answers.ElementAt(1).FieldId.Should().Be("field-two");
        filing2.Answers.ElementAt(1).Answer.Should().Be("Updated Answer 2");
        filing2.Answers.ElementAt(2).FieldId.Should().Be("field-three");
        filing2.Answers.ElementAt(2).Answer.Should().Be("Option C");
    }

    private void SetupSheet()
    {
        var testSheetField1 = new SheetField
        {
            SheetId = "test-sheet",
            Id = "field-one",
            Sequence = 1,
            NameZh = "字段一",
            NameEn = "Field One",
            Kind = SheetFieldKind.ShortText,
        };
        var testSheetField2 = new SheetField
        {
            SheetId = "test-sheet",
            Id = "field-two",
            Sequence = 2,
            NameZh = "字段二",
            NameEn = "Field Two",
            Kind = SheetFieldKind.LongText,
        };
        var testSheetField3 = new SheetField
        {
            SheetId = "test-sheet",
            Id = "field-three",
            Sequence = 3,
            NameZh = "字段三",
            NameEn = "Field Three",
            Kind = SheetFieldKind.SingleChoice,
            SingleChoiceOptions = ["Option A", "Option B", "Option C"]
        };
        var testSheetField4 = new SheetField
        {
            SheetId = "test-sheet",
            Id = "field-four",
            Sequence = 4,
            NameZh = "字段四",
            NameEn = "Field Four",
            Kind = SheetFieldKind.ShortText,
            IsDeleted = true,
        };
        IList<SheetField> sheetFields = [testSheetField1, testSheetField2, testSheetField3, testSheetField4];
        dbContext.SheetField.AddRange(sheetFields);

        var testSheet = new Sheet { Id = "test-sheet", Name = "Test Sheet" };
        dbContext.Sheet.Add(testSheet);

        dbContext.SaveChanges();
    }

    private void SetupUser()
    {
        user = new User
        {
            Id = Ulid.NewUlid(),
            Cid = "1234567",
            FullName = "Alice Smith",
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow,
        };
        dbContext.User.Add(user);
        dbContext.SaveChanges();
    }

    private Ulid SetupSheetFiling()
    {
        var id = Ulid.NewUlid();
        var testSheetFiling = new SheetFiling
        {
            Id = id,
            SheetId = "test-sheet",
            UserId = user!.Id,
            FiledAt = DateTimeOffset.UtcNow,
        };
        dbContext.SheetFiling.Add(testSheetFiling);

        dbContext.SaveChanges();

        return id;
    }
}
