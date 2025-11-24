using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/applications")]
[Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
public class AtcApplicationController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor,
    AtcApplicationService atcApplicationService
) : Controller
{
    protected const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        var applications = await atcApplicationService.GetApplications();
        return applications.Select(app => new AtcApplicationSummaryDto(app));
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.AtcApplicationNotFound>]
    public async Task<AtcApplicationDto> GetById(Ulid id)
    {
        var application = await atcApplicationService.GetApplication(id);

        if (application == null)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        return new(application);
    }

    [HttpGet("review-sheet")]
    public async Task<SheetDto> GetSheet()
    {
        await sheetService.EnsureSheetAsync(ATC_APPLICATION_REVIEW_SHEET_ID, "ATC Application Review Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_REVIEW_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application review sheet is not configured.");
        return new SheetDto(sheet);
    }

    [HttpPut("{id}/review")]
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

        var reviewFiling = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_REVIEW_SHEET_ID,
            application.ReviewFiling?.Id,
            userId,
            reviewDto.ReviewAnswers.ToDictionary(kv => kv.Id, kv => kv.Answer),
            CancellationToken.None);

        application.ReviewFilingId = reviewFiling.Id;

        await database.SaveChangesAsync();

        return new AtcApplicationDto(application);
    }

    [HttpPut("{id}")]
    public async Task<AtcApplicationDto> UpdateStatus(
        Ulid id,
        [FromBody] AtcApplicationUpdateRequest updateDto)
    {
        var application = await atcApplicationService.GetApplication(id);

        if (application == null)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        application.Status = updateDto.Status;
        await database.SaveChangesAsync();
        return new AtcApplicationDto(application);
    }
}
