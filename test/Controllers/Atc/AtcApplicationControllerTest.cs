using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Diagnostics;
using Moq;
using Net.Vatprc.Uniapi.Controllers.Atc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Test;
using Net.Vatprc.Uniapi.Utils;
using static Net.Vatprc.Uniapi.Controllers.UserAtcApplicationController;
namespace Net.Vatprc.Uniapi.Tests.Controllers.Atc;

[TestFixture]
public class AtcApplicationControllerTest : TestWithDatabase
{
    private const string ATC_APPLICATION_SHEET_ID = "atc-application";
    private const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    private AtcApplicationController controller = null!;
    private SheetService realSheetService = null!;
    private Mock<SheetService> sheetService = null!;
    private Mock<IUserAccessor> userAccessor = null!;
    private Ulid applicantUserId;
    private Ulid adminUserId;

    [SetUp]
    public async Task Setup()
    {
        realSheetService = new SheetService(dbContext);
        sheetService = new Mock<SheetService>(dbContext);
        userAccessor = new Mock<IUserAccessor>();
        controller = new AtcApplicationController(dbContext, sheetService.Object, userAccessor.Object);

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

        applicantUserId = u1.Id;
        adminUserId = u2.Id;
        userAccessor.Setup(ua => ua.GetUserId()).Returns(u2.Id);

        await realSheetService.EnsureSheetAsync(ATC_APPLICATION_SHEET_ID, "ATC Application Sheet");
        await realSheetService.EnsureSheetAsync(ATC_APPLICATION_REVIEW_SHEET_ID, "ATC Application Review Sheet");
        await realSheetService.SetSheetFieldsAsync(ATC_APPLICATION_SHEET_ID,
        [
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
        ]);
        await realSheetService.SetSheetFieldsAsync(ATC_APPLICATION_REVIEW_SHEET_ID,
        [
            new SheetField
            {
                Id = "review",
                SheetId = ATC_APPLICATION_REVIEW_SHEET_ID,
                Sequence = 1,
                NameZh = "面试评价",
                NameEn = "Review",
                Kind = SheetFieldKind.LongText,
            },
        ]);
    }

    [TearDown]
    public void TearDown()
    {
        controller.Dispose();
    }

    [Test]
    public async Task List_ReturnsAllApplications()
    {
        // Arrange
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            applicantUserId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "2 years" },
            });

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = applicantUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
        };

        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        // Act
        var result = await controller.List();

        // Assert
        var appDto = result.First();
        result.Count().Should().Be(1);
        appDto.Id.Should().Be(application.Id);
        appDto.UserId.Should().Be(applicantUserId);
    }

    [Test]
    public async Task GetById_ReturnsApplicationDetails()
    {
        // Arrange
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            applicantUserId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "2 years" },
            });

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = applicantUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
        };

        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        // Act
        var appDto = await controller.GetById(application.Id);

        // Assert
        appDto.Id.Should().Be(application.Id);
        appDto.UserId.Should().Be(applicantUserId);
        appDto.ApplicationFilingAnswers.Count().Should().Be(2);
        var nameAnswer = appDto.ApplicationFilingAnswers.First(a => a.Field.Id == "full-name");
        nameAnswer.Answer.Should().Be("Alice");
        var experienceAnswer = appDto.ApplicationFilingAnswers.First(a => a.Field.Id == "experience");
        experienceAnswer.Answer.Should().Be("2 years");
    }

    [Test]
    public async Task GetById_ReturnsApplicationDetailsWithReview()
    {
        // Arrange
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            applicantUserId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "2 years" },
            });

        var reviewFiling = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_REVIEW_SHEET_ID,
            null,
            adminUserId,
            new Dictionary<string, string>
            {
                { "review", "Excellent candidate." },
            });

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = applicantUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
            ReviewFilingId = reviewFiling.Id,
        };

        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        // Act
        var appDto = await controller.GetById(application.Id);

        // Assert
        appDto.Id.Should().Be(application.Id);
        appDto.UserId.Should().Be(applicantUserId);
        appDto.ApplicationFilingAnswers.Count().Should().Be(2);
        var nameAnswer = appDto.ApplicationFilingAnswers.First(a => a.Field.Id == "full-name");
        nameAnswer.Answer.Should().Be("Alice");
        var experienceAnswer = appDto.ApplicationFilingAnswers.First(a => a.Field.Id == "experience");
        experienceAnswer.Answer.Should().Be("2 years");
        appDto.ReviewFilingAnswers.Should().NotBeNull();
        appDto.ReviewFilingAnswers.Count().Should().Be(1);
        var reviewAnswer = appDto.ReviewFilingAnswers.First(a => a.Field.Id == "review");
        reviewAnswer.Answer.Should().Be("Excellent candidate.");
    }

    [Test]
    public void GetById_NonExistentApplication_ThrowsNotFound()
    {
        // Act & Assert
        var nonExistentId = Ulid.NewUlid();
        var ex = Assert.ThrowsAsync<ApiError.AtcApplicationNotFound>(async () => await controller.GetById(nonExistentId));
    }

    [Test]
    public async Task GetSheet_ReturnsReviewSheet()
    {
        // Act
        var sheetDto = await controller.GetSheet();

        // Assert
        sheetDto.Id.Should().Be(ATC_APPLICATION_REVIEW_SHEET_ID);
        sheetDto.Fields.Should().HaveCount(1);
    }

    [Test]
    public async Task ReviewApplication_UpdatesReviewFiling()
    {
        // Arrange
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            applicantUserId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "2 years" },
            });

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = applicantUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
        };

        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        var reviewDto = new AtcApplicationController.AtcApplicationReviewDto
        {
            Answers = [
                new AtcApplicationController.AtcApplicationReviewAnswerDto
                {
                    Id = "review",
                    Answer = "Strong candidate.",
                },
            ],
        };

        // Act
        var updatedAppDto = await controller.ReviewApplication(application.Id, reviewDto);

        // Assert
        updatedAppDto.ReviewFilingAnswers.Should().NotBeNull();
        updatedAppDto.ReviewFilingAnswers.Count().Should().Be(1);
        var reviewAnswer = updatedAppDto.ReviewFilingAnswers.First(a => a.Field.Id == "review");
        reviewAnswer.Answer.Should().Be("Strong candidate.");
    }

    [Test]
    public void ReviewApplication_NonExistentApplication_ThrowsNotFound()
    {
        // Arrange
        var nonExistentId = Ulid.NewUlid();
        var reviewDto = new AtcApplicationController.AtcApplicationReviewDto
        {
            Answers = [
                new AtcApplicationController.AtcApplicationReviewAnswerDto
                {
                    Id = "review",
                    Answer = "Strong candidate.",
                },
            ],
        };

        // Act & Assert
        var ex = Assert.ThrowsAsync<ApiError.AtcApplicationNotFound>(async () => await controller.ReviewApplication(nonExistentId, reviewDto));
    }

    [Test]
    public async Task UpdateStatus_ChangesApplicationStatus()
    {
        // Arrange
        var filing = await realSheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            applicantUserId,
            new Dictionary<string, string>
            {
                { "full-name", "Alice" },
                { "experience", "2 years" },
            });

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = applicantUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
        };

        dbContext.AtcApplication.Add(application);
        await dbContext.SaveChangesAsync();

        // Act
        var updatedAppDto = await controller.UpdateStatus(application.Id, new()
        {
            Status = AtcApplicationStatusDto.Approved,
        });

        // Assert
        updatedAppDto.Status.Should().Be(AtcApplicationStatusDto.Approved);
    }
}
