using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using static Net.Vatprc.Uniapi.Services.SheetService;

namespace Net.Vatprc.Uniapi.Test.Services;

[TestFixture]
public class SheetServiceTest : TestWithDatabase
{
    private SheetService sheetService;

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
}
