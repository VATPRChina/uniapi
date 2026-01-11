using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters.EmailAdapter;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/applications")]
public class AtcApplicationController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor,
    AtcApplicationService atcApplicationService,
    ISmtpEmailAdapter emailAdapter
) : Controller
{
    public const string ATC_APPLICATION_SHEET_ID = "atc-application";
    public const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);
        var applications = await atcApplicationService.GetApplications();
        return applications
            .Where(app => isAdmin || app.UserId == userAccessor.GetUserId())
            .Select(app => AtcApplicationSummaryDto.From(app, isAdmin, userAccessor.GetUserId()));
    }

    [HttpPost]
    public async Task<AtcApplicationSummaryDto> Create(AtcApplicationRequest req)
    {
        var curUserId = userAccessor.GetUserId();

        var existingApplicationCount = await database.AtcApplication
            .CountAsync(app => app.UserId == curUserId && app.Status != AtcApplicationStatus.Rejected);
        if (existingApplicationCount > 0)
        {
            throw new ApiError.AtcApplicationAlreadyExists();
        }

        var filing = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            curUserId,
            req.RequestAnswers.ToDictionary(
                answer => answer.Id,
                answer => answer.Answer));

        var application = new AtcApplication
        {
            Id = Ulid.NewUlid(),
            UserId = curUserId,
            AppliedAt = DateTimeOffset.UtcNow,
            ApplicationFilingId = filing.Id,
        };

        database.AtcApplication.Add(application);
        await database.SaveChangesAsync();

        return AtcApplicationSummaryDto.From(application, false, curUserId);
    }

    protected async Task<AtcApplication> GetApplication(Ulid id)
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);
        var application = await atcApplicationService.GetApplication(id);

        if (application == null)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }
        if (!isAdmin && application.UserId != userAccessor.GetUserId())
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        return application;
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.AtcApplicationNotFound>]
    public async Task<AtcApplicationDto> GetById(Ulid id)
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);
        var application = await atcApplicationService.GetApplication(id);

        if (application == null)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }
        if (!isAdmin && application.UserId != userAccessor.GetUserId())
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        return AtcApplicationDto.From(application, isAdmin, userAccessor.GetUserId());
    }

    [HttpPut("{id}")]
    [ApiError.Has<ApiError.AtcApplicationNotFound>]
    [ApiError.Has<ApiError.AtcApplicationCannotUpdate>]
    public async Task<AtcApplicationDto> Update(Ulid id, AtcApplicationRequest req)
    {
        var curUserId = userAccessor.GetUserId();

        var application = await GetApplication(id);

        if (application.Status != AtcApplicationStatus.Submitted)
        {
            throw new ApiError.AtcApplicationCannotUpdate(id, application.Status);
        }

        var filing = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            application.ApplicationFilingId,
            curUserId,
            req.RequestAnswers.ToDictionary(
                answer => answer.Id,
                answer => answer.Answer));

        return AtcApplicationDto.From(application, false, curUserId);
    }

    [HttpGet("sheet")]
    public async Task<SheetDto> GetApplySheet()
    {
        await sheetService.EnsureSheetAsync(ATC_APPLICATION_SHEET_ID, "ATC Application Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application sheet is not configured.");
        return SheetDto.From(sheet);
    }

    [HttpGet("review-sheet")]
    public async Task<SheetDto> GetSheet()
    {
        await sheetService.EnsureSheetAsync(ATC_APPLICATION_REVIEW_SHEET_ID, "ATC Application Review Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_REVIEW_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application review sheet is not configured.");
        return SheetDto.From(sheet);
    }

    [HttpPut("{id}/review")]
    [Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
    public async Task<AtcApplicationDto> ReviewApplication(
        Ulid id,
        AtcApplicationReviewRequest reviewDto)
    {
        var userId = userAccessor.GetUserId();
        var application = await atcApplicationService.GetApplication(id);

        if (application == null)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        application.Status = reviewDto.Status;

        var reviewFiling = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_REVIEW_SHEET_ID,
            application.ReviewFiling?.Id,
            userId,
            reviewDto.ReviewAnswers.ToDictionary(kv => kv.Id, kv => kv.Answer),
            CancellationToken.None);

        application.ReviewFilingId = reviewFiling.Id;

        await database.SaveChangesAsync();

        var userEmail = application.User?.Email;
        if (userEmail != null)
        {
            await emailAdapter.SendEmailAsync(
                userEmail,
                new AtcApplicationStatusChangeEmail(application),
                CancellationToken.None);
        }

        return AtcApplicationDto.From(application, true, userId);
    }
}
