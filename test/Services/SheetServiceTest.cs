using AwesomeAssertions;
using Microsoft.EntityFrameworkCore;
using MockQueryable.Moq;
using Moq;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Test.Services;

[TestFixture]
public class SheetServiceTest
{
    private Mock<Database> dbContext;
    private SheetService sheetService;

    [SetUp]
    public void Setup()
    {
        dbContext = new Mock<Database>();

        sheetService = new SheetService(dbContext.Object);
    }

    [Test]
    public async Task GetSheetByIdAsync_SheetExists_ReturnsSheet()
    {
        SetupSheet();

        var result = await sheetService.GetSheetByIdAsync("test-sheet");

        result.Should().NotBeNull();
        result.Id.Should().Be("test-sheet");
        result.Fields.Should().HaveCount(3);
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
            Sequence = 1,
            NameZh = "字段一",
            NameEn = "Field One",
            Kind = SheetFieldKind.ShortText,
        };
        var testSheetField2 = new SheetField
        {
            SheetId = "test-sheet",
            Sequence = 2,
            NameZh = "字段二",
            NameEn = "Field Two",
            Kind = SheetFieldKind.LongText,
        };
        var testSheetField3 = new SheetField
        {
            SheetId = "test-sheet",
            Sequence = 3,
            NameZh = "字段三",
            NameEn = "Field Three",
            Kind = SheetFieldKind.SingleChoice,
        };
        ICollection<SheetField> sheetFields = [testSheetField1, testSheetField2, testSheetField3];
        var testSheet = new Sheet { Id = "test-sheet", Name = "Test Sheet", Fields = sheetFields };
        ICollection<Sheet> sheets = [testSheet];
        dbContext.SetupGet(d => d.Sheet).Returns(sheets.BuildMockDbSet().Object);
    }
}
