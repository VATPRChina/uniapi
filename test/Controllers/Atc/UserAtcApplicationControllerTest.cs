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
    private const string ATC_APPLICATION_SHEET_ID = "atc-application";
    private const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    private UserAtcApplicationController controller = null!;
    private SheetService realSheetService = null!;
    private Mock<SheetService> sheetService = null!;
    private Mock<IUserAccessor> userAccessor = null!;
    private Ulid userId;
    private Ulid user2Id;

    [SetUp]
    public async Task Setup()
    {
        realSheetService = new SheetService(dbContext);
        sheetService = new Mock<SheetService>(dbContext);
        userAccessor = new Mock<IUserAccessor>();
        controller = new UserAtcApplicationController(dbContext, sheetService.Object, userAccessor.Object);

        var u1 = new User
        {
            Id = Ulid.NewUlid(),
            Cid = "100",
            FullName = "Alice",
            Email = "alice@example.com",
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow
        };
        var u2 = new User
        {
            Id = Ulid.NewUlid(),
            Cid = "101",
            FullName = "Bob",
            Email = "bob@example.com",
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow
        };
        dbContext.User.AddRange(u1, u2);
        await dbContext.SaveChangesAsync();

        userId = u1.Id;
        user2Id = u2.Id;
        userAccessor.Setup(ua => ua.GetUserId()).Returns(userId);

        await realSheetService.EnsureSheetAsync(ATC_APPLICATION_SHEET_ID, "ATC Application Sheet");
        await realSheetService.EnsureSheetAsync(ATC_APPLICATION_REVIEW_SHEET_ID, "ATC Application Review Sheet");
        await realSheetService.SetSheetFieldsAsync(ATC_APPLICATION_SHEET_ID, new[]
        {
            new SheetField
            {
                Id = "full-name",
                SheetId = ATC_APPLICATION_SHEET_ID,
                Sequence = 1,
                NameZh = "全名",
                NameEn = "Full Name",
                Kind = SheetFieldKind.ShortText,
            },
            new SheetField
            {
                Id = "cid",
                SheetId = ATC_APPLICATION_SHEET_ID,
                Sequence = 2,
                NameZh = "CID",
                NameEn = "CID",
                Kind = SheetFieldKind.ShortText,
                IsDeleted = true,
            },
            new SheetField
            {
                Id = "experience",
                SheetId = ATC_APPLICATION_SHEET_ID,
                Sequence = 3,
                NameZh = "ATC经验",
                NameEn = "ATC Experience",
                Kind = SheetFieldKind.LongText,
            },
        });
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
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            userId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "I have been an ATC for 2 years." },
            },
            CancellationToken.None);
        var app1 = new AtcApplication { Id = Ulid.NewUlid(), UserId = userId, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-5), ApplicationFilingId = filing.Id };
        var app2 = new AtcApplication { Id = Ulid.NewUlid(), UserId = user2Id, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-1), ApplicationFilingId = filing.Id };
        var app3 = new AtcApplication { Id = Ulid.NewUlid(), UserId = userId, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-2), ApplicationFilingId = filing.Id };
        dbContext.AtcApplication.AddRange(app1, app2, app3);
        dbContext.SaveChanges();
        userAccessor.Setup(ua => ua.GetUserId()).Returns(userId);

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
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            userId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "I have been an ATC for 2 years." },
            },
            CancellationToken.None);
        var app = new AtcApplication { Id = Ulid.NewUlid(), UserId = userId, AppliedAt = DateTimeOffset.UtcNow.AddMinutes(-5), ApplicationFilingId = filing.Id };
        dbContext.AtcApplication.Add(app);
        await dbContext.SaveChangesAsync();

        // Act
        var dto = await controller.GetById(app.Id);

        // Assert
        dto.Id.Should().Be(app.Id);
        dto.ApplicationFilingAnswers.Should().HaveCount(2);
        var name = dto.ApplicationFilingAnswers.First(a => a.Field.Id == "full-name");
        name.Answer.Should().Be("Alice");
        var experience = dto.ApplicationFilingAnswers.First(a => a.Field.Id == "experience");
        experience.Answer.Should().Be("I have been an ATC for 2 years.");
    }

    [Test]
    public async Task GetSheet_ReturnsSheetDto()
    {
        // Act
        var sheetDto = await controller.GetSheet();

        // Assert
        sheetDto.Id.Should().Be(ATC_APPLICATION_SHEET_ID);
        sheetDto.Fields.Should().HaveCount(2);
    }

    [Test]
    public async Task Create_CreatesApplicationAndFiling()
    {
        // Arrange
        var req = new UserAtcApplicationController.AtcApplicationCreateDto
        {
            ApplicationFilingAnswers = [
                new () { Id = "full-name", Answer = "Answer A" },
                new () { Id = "experience", Answer = "Answer B" },
            ],
        };

        // Act
        var summary = await controller.Create(req);

        // Assert
        summary.UserId.Should().Be(userId);

        var stored = dbContext.AtcApplication.Find(summary.Id);
        stored.Should().NotBeNull();
        var filing = dbContext.SheetFiling.Find(stored!.ApplicationFilingId);
        filing.Should().NotBeNull();
        dbContext.SheetFilingAnswer.Where(a => a.FilingId == filing!.Id).Should().HaveCount(2);
    }

    [Test]
    public async Task Update_UpdatesExistingFiling()
    {
        // Arrange
        var initialFiling = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            userId,
            new Dictionary<string, string>
            {
                { "full-name", "Initial Name" },
                { "experience", "Initial Experience" },
            },
            CancellationToken.None);
        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = userId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = initialFiling.Id,
        };
        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        var req = new UserAtcApplicationController.AtcApplicationCreateDto
        {
            ApplicationFilingAnswers = [
                new () { Id = "full-name", Answer = "Updated Name" },
                new () { Id = "experience", Answer = "Updated Experience" },
            ],
        };

        // Act
        var dto = await controller.Update(application.Id, req);

        // Assert
        dto.Id.Should().Be(application.Id);

        var updatedFiling = await realSheetService.GetSheetFilingByIdAsync(initialFiling.Id);
        updatedFiling.Should().NotBeNull();
        var answers = updatedFiling!.Answers.ToDictionary(a => a.FieldId, a => a);
        answers["full-name"].Answer.Should().Be("Updated Name");
        answers["experience"].Answer.Should().Be("Updated Experience");
    }
}
