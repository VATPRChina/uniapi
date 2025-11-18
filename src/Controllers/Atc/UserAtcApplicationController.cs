using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("api/users/me/atc/applications")]
public class UserAtcApplicationController(
    Database database,
    SheetService sheetService,
    UserAccessor userAccessor
) : Controller
{
    protected const string ATC_APPLICATION_SHEET_ID = "atc-application";
    protected const string ATC_APPLICATION_REVIEW_SHEET_ID = "atc-application-review";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        var curUserId = userAccessor.GetUserId();
        return await database.AtcApplication
            .Where(app => app.UserId == curUserId)
            .OrderByDescending(app => app.AppliedAt)
            .Select(app => new AtcApplicationSummaryDto(app))
            .ToListAsync();
    }

    [HttpGet("{id}")]
    public async Task<AtcApplicationDto> List(Ulid id)
    {
        var curUserId = userAccessor.GetUserId();
        var application = await database.AtcApplication
            .Where(a => a.UserId == curUserId && a.Id == id)
            .Include(a => a.ApplicationFiling)
                .ThenInclude(af => af!.Answers)
                    .ThenInclude(ans => ans.Field)
            .Include(a => a.ReviewFiling)
                .ThenInclude(rf => rf!.Answers)
                    .ThenInclude(ans => ans.Field)
            .Select(a => new AtcApplicationDto(a))
            .SingleOrDefaultAsync() ??
            throw new ApiError.AtcApplicationNotFound(id);

        return application;
    }

    [HttpGet("sheet")]
    public async Task<AtcApplicationSheetDto> GetSheet()
    {
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application sheet is not configured.");
        return new AtcApplicationSheetDto(sheet);
    }

    [HttpPost]
    public async Task<AtcApplicationSummaryDto> Create(AtcApplicationCreateDto req)
    {
        var curUserId = userAccessor.GetUserId();

        var filing = await sheetService.CreateSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            curUserId,
            req.SheetAnswers,
            HttpContext.RequestAborted);

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

    public record class AtcApplicationSummaryDto(
        Ulid Id,
        Ulid UserId,
        DateTimeOffset AppliedAt)
    {
        public AtcApplicationSummaryDto(AtcApplication application) : this(
            application.Id,
            application.UserId,
            application.AppliedAt)
        { }
    }

    public record class AtcApplicationDto(
        Ulid Id,
        Ulid UserId,
        DateTimeOffset AppliedAt,
        IEnumerable<AtcApplicationField> ApplicationFilingAnswers,
        IEnumerable<AtcApplicationField>? ReviewFilingAnswers = null)
    {
        public AtcApplicationDto(AtcApplication application) : this(
            application.Id,
            application.UserId,
            application.AppliedAt,
            application.ApplicationFiling?.Answers.Select(answer => new AtcApplicationField(answer)) ??
                throw new ArgumentNullException(nameof(application), "ApplicationFiling must be loaded"),
            application.ReviewFiling?.Answers.Select(answer => new AtcApplicationField(answer)))
        { }
    }

    public record class AtcApplicationField(
        string Name,
        string Answer)
    {
        public AtcApplicationField(SheetFilingAnswer answer) : this(
            answer.Field?.NameZh
                ?? throw new ArgumentNullException(nameof(answer), "answer.Field must be loaded"),
            answer.Answer)
        { }
    }

    public record class AtcApplicationSheetDto(
        string Id,
        string Name,
        IEnumerable<AtcApplicationSheetFieldDto> Fields
    )
    {
        public AtcApplicationSheetDto(Sheet sheet) : this(
            sheet.Id,
            sheet.Name,
            sheet.Fields.Select(field => new AtcApplicationSheetFieldDto(field)))
        { }
    }

    public record class AtcApplicationSheetFieldDto(
        string SheetId,
        string Id,
        uint Sequence,
        string NameZh,
        string? NameEn,
        SheetFieldKind Kind,
        IEnumerable<string> SingleChoiceOptions,
        bool IsDeleted)
    {
        public AtcApplicationSheetFieldDto(SheetField field) : this(
            field.SheetId,
            field.Id,
            field.Sequence,
            field.NameZh,
            field.NameEn,
            field.Kind,
            field.SingleChoiceOptions,
            field.IsDeleted)
        { }
    }

    public record class AtcApplicationCreateDto
    {
        public required IDictionary<string, string> SheetAnswers { get; set; }
    }
}
