using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController]
[Route("api/users/me/atc/applications")]
public class UserAtcApplicationController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor,
    AtcApplicationService atcApplicationService
) : Controller
{
    protected const string ATC_APPLICATION_SHEET_ID = "atc-application";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        var curUserId = userAccessor.GetUserId();
        var applications = await atcApplicationService.GetApplications(curUserId);
        return applications.Select(app => new AtcApplicationSummaryDto(app));
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.AtcApplicationNotFound>]
    public async Task<AtcApplicationDto> GetById(Ulid id)
    {
        var application = await GetApplication(id);

        return new(application);
    }

    [HttpGet("sheet")]
    public async Task<SheetDto> GetSheet()
    {
        await sheetService.EnsureSheetAsync(ATC_APPLICATION_SHEET_ID, "ATC Application Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application sheet is not configured.");
        return new SheetDto(sheet);
    }

    [HttpPost]
    public async Task<AtcApplicationSummaryDto> Create(AtcApplicationRequest req)
    {
        var curUserId = userAccessor.GetUserId();

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

        return new AtcApplicationSummaryDto(application);
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

        return new AtcApplicationDto(application);
    }

    protected async Task<AtcApplication> GetApplication(Ulid id)
    {
        var curUserId = userAccessor.GetUserId();
        var application = await atcApplicationService.GetApplication(id) ??
            throw new ApiError.AtcApplicationNotFound(id);

        if (application.UserId != curUserId)
        {
            throw new ApiError.AtcApplicationNotFound(id);
        }

        return application;
    }
}
