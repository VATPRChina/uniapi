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
    IUserAccessor userAccessor
) : Controller
{
    protected const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        return await database.AtcApplication
            .Include(a => a.User)
            .OrderByDescending(app => app.AppliedAt)
            .Select(app => new AtcApplicationSummaryDto(app))
            .ToListAsync();
    }

    [HttpGet("{id}")]
    public async Task<AtcApplicationDto> GetById(Ulid id)
    {
        var application = await database.AtcApplication
            .Where(a => a.Id == id)
            .Include(a => a.User)
            .Include(a => a.ApplicationFiling)
                .ThenInclude(af => af!.Answers)
                    .ThenInclude(ans => ans.Field)
            .Include(a => a.ReviewFiling)
                .ThenInclude(rf => rf!.Answers)
                    .ThenInclude(ans => ans.Field)
            .OrderByDescending(a => a.AppliedAt)
            .Select(a => new AtcApplicationDto(a))
            .SingleOrDefaultAsync() ??
            throw new ApiError.AtcApplicationNotFound(id);

        return application;
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
        var application = await database.AtcApplication
            .Where(a => a.Id == id)
            .Include(a => a.User)
            .Include(a => a.ReviewFiling)
                .ThenInclude(rf => rf!.Answers)
                    .ThenInclude(ans => ans.Field)
            .SingleOrDefaultAsync() ??
            throw new ApiError.AtcApplicationNotFound(id);

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
        var application = await database.AtcApplication
            .Where(a => a.Id == id)
            .Include(a => a.User)
            .Include(a => a.ApplicationFiling)
                .ThenInclude(af => af!.Answers)
                    .ThenInclude(ans => ans.Field)
            .Include(a => a.ReviewFiling)
                .ThenInclude(rf => rf!.Answers)
                    .ThenInclude(ans => ans.Field)
            .SingleOrDefaultAsync() ??
            throw new ApiError.AtcApplicationNotFound(id);

        application.Status = updateDto.Status;
        await database.SaveChangesAsync();
        return new AtcApplicationDto(application);
    }
}
