using System.Security.Claims;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.DependencyInjection;
using Moq;
using Net.Vatprc.Uniapi.Controllers;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Test.Controllers.Atc;

[TestFixture]
public class UserAtcApplicationControllerTest : TestWithDatabase
{
    private UserAtcApplicationController controller = null!;
    private Mock<SheetService> sheetService = null!;
    private Mock<IUserAccessor> userAccessor = null!;
    private Ulid user1Id;
    private Ulid user2Id;

    [SetUp]
    public void Setup()
    {
        sheetService = new Mock<SheetService>(dbContext);
        userAccessor = new Mock<IUserAccessor>();
        controller = new UserAtcApplicationController(dbContext, sheetService.Object, userAccessor.Object);

        var u1 = new User { Id = Ulid.NewUlid(), Cid = "100", FullName = "Alice", Email = "alice@example.com", CreatedAt = DateTimeOffset.UtcNow, UpdatedAt = DateTimeOffset.UtcNow };
        var u2 = new User { Id = Ulid.NewUlid(), Cid = "101", FullName = "Bob", Email = "bob@example.com", CreatedAt = DateTimeOffset.UtcNow, UpdatedAt = DateTimeOffset.UtcNow };
        dbContext.User.AddRange(u1, u2);
        dbContext.SaveChanges();

        user1Id = u1.Id;
        user2Id = u2.Id;
    }

    [TearDown]
    public void TearDown()
    {
        controller.Dispose();
    }

    [Test]
    public async Task List_ReturnsOnlyCurrentUserApplications()
    {
        // Arrange
        var sheet = new Sheet { Id = "atc-application", Name = "ATC Application" };
        dbContext.Sheet.Add(sheet);
        var filing = new SheetFiling { Id = Ulid.NewUlid(), SheetId = "atc-application", UserId = user1Id, FiledAt = DateTimeOffset.UtcNow };
        dbContext.SheetFiling.Add(filing);
        var app1 = new AtcApplication { Id = Ulid.NewUlid(), UserId = user1Id, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-5), ApplicationFilingId = filing.Id };
        var app2 = new AtcApplication { Id = Ulid.NewUlid(), UserId = user2Id, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-1), ApplicationFilingId = filing.Id };
        var app3 = new AtcApplication { Id = Ulid.NewUlid(), UserId = user1Id, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-2), ApplicationFilingId = filing.Id };
        dbContext.AtcApplication.AddRange(app1, app2, app3);
        dbContext.SaveChanges();
        userAccessor.Setup(ua => ua.GetUserId()).Returns(user1Id);

        // Act
        var list = (await controller.List()).ToList();

        // Assert
        list.Should().HaveCount(2);
        list.Select(x => x.Id).Should().Contain([app1.Id, app3.Id]);
    }

    [Test]
    public async Task GetById_ReturnsApplicationWithAnswers()
    {
        // Arrange
        var u = new User { Id = Ulid.NewUlid(), Cid = "200", FullName = "Charlie", CreatedAt = DateTimeOffset.UtcNow, UpdatedAt = DateTimeOffset.UtcNow };
        dbContext.User.Add(u);

        // create sheet field and filing/answers
        var field = new SheetField { SheetId = "atc-application", Id = "f1", Sequence = 1, NameZh = "字段", NameEn = "Field", Kind = SheetFieldKind.ShortText };
        dbContext.SheetField.Add(field);
        var filing = new SheetFiling { Id = Ulid.NewUlid(), SheetId = "atc-application", UserId = u.Id, FiledAt = DateTimeOffset.UtcNow };
        dbContext.SheetFiling.Add(filing);
        var answer = new SheetFilingAnswer { SheetId = "atc-application", FieldId = "f1", FilingId = filing.Id, Field = field, Answer = "Ans" };
        dbContext.SheetFilingAnswer.Add(answer);

        var app = new AtcApplication { Id = Ulid.NewUlid(), UserId = u.Id, AppliedAt = DateTimeOffset.UtcNow, ApplicationFilingId = filing.Id };
        dbContext.AtcApplication.Add(app);

        dbContext.SaveChanges();

        // Act
        var dto = await controller.List(app.Id);

        // Assert
        dto.Id.Should().Be(app.Id);
        dto.ApplicationFilingAnswers.Should().HaveCount(1);
        dto.ApplicationFilingAnswers.First().Answer.Should().Be("Ans");
    }

    [Test]
    public async Task GetSheet_ReturnsSheetDto()
    {
        // Arrange
        var field1 = new SheetField { SheetId = "atc-application", Id = "s1", Sequence = 1, NameZh = "字段一", Kind = SheetFieldKind.ShortText };
        var field2 = new SheetField { SheetId = "atc-application", Id = "s2", Sequence = 2, NameZh = "字段二", Kind = SheetFieldKind.ShortText };
        dbContext.Sheet.Add(new Sheet { Id = "atc-application", Name = "ATC Application", Fields = new List<SheetField> { field1, field2 } });
        dbContext.SaveChanges();

        // Act
        var sheetDto = await controller.GetSheet();

        // Assert
        sheetDto.Id.Should().Be("atc-application");
        sheetDto.Fields.Should().HaveCount(2);
    }

    [Test]
    public async Task Create_CreatesApplicationAndFiling()
    {
        // Arrange
        var u = new User { Id = Ulid.NewUlid(), Cid = "400", FullName = "Eve", CreatedAt = DateTimeOffset.UtcNow, UpdatedAt = DateTimeOffset.UtcNow };
        dbContext.User.Add(u);

        var field1 = new SheetField { SheetId = "atc-application", Id = "fa", Sequence = 1, NameZh = "A", Kind = SheetFieldKind.ShortText };
        var field2 = new SheetField { SheetId = "atc-application", Id = "fb", Sequence = 2, NameZh = "B", Kind = SheetFieldKind.ShortText };
        dbContext.Sheet.Add(new Sheet { Id = "atc-application", Name = "ATC Application", Fields = new List<SheetField> { field1, field2 } });
        dbContext.SaveChanges();

        var req = new UserAtcApplicationController.AtcApplicationCreateDto
        {
            SheetAnswers = new Dictionary<string, string>
            {
                { "fa", "hello" },
                { "fb", "world" },
            }
        };

        // Act
        var summary = await controller.Create(req);

        // Assert
        summary.UserId.Should().Be(u.Id);

        var stored = dbContext.AtcApplication.Find(summary.Id);
        stored.Should().NotBeNull();
        var filing = dbContext.SheetFiling.Find(stored!.ApplicationFilingId);
        filing.Should().NotBeNull();
        dbContext.SheetFilingAnswer.Where(a => a.FilingId == filing!.Id).Should().HaveCount(2);
    }
}
