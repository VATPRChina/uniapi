using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Sheet;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using static Net.Vatprc.Uniapi.Controllers.Auth.UserController;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("api/users/me/atc/applications")]
public class UserAtcApplicationController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor
) : Controller
{
    protected const string ATC_APPLICATION_SHEET_ID = "atc-application";

    [HttpGet]
    public async Task<IEnumerable<AtcApplicationSummaryDto>> List()
    {
        var curUserId = userAccessor.GetUserId();
        return await database.AtcApplication
            .Where(app => app.UserId == curUserId)
            .Include(a => a.User)
            .OrderByDescending(app => app.AppliedAt)
            .Select(app => new AtcApplicationSummaryDto(app))
            .ToListAsync();
    }

    [HttpGet("{id}")]
    public async Task<AtcApplicationDto> GetById(Ulid id)
    {
        var curUserId = userAccessor.GetUserId();
        var application = await database.AtcApplication
            .Where(a => a.UserId == curUserId && a.Id == id)
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

    [HttpGet("sheet")]
    public async Task<AtcApplicationSheetDto> GetSheet()
    {
        await sheetService.EnsureSheetAsync(ATC_APPLICATION_SHEET_ID, "ATC Application Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(ATC_APPLICATION_SHEET_ID)
            ?? throw new InvalidOperationException("ATC application sheet is not configured.");
        return new AtcApplicationSheetDto(sheet);
    }

    [HttpPost]
    public async Task<AtcApplicationDto> Create(AtcApplicationCreateDto req)
    {
        var curUserId = userAccessor.GetUserId();

        var filing = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            null,
            curUserId,
            req.ApplicationFilingAnswers.ToDictionary(
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

        return new AtcApplicationDto(application);
    }

    [HttpPut("{id}")]
    public async Task<AtcApplicationDto> Update(Ulid id, AtcApplicationCreateDto req)
    {
        var curUserId = userAccessor.GetUserId();

        var application = await database.AtcApplication
            .Where(a => a.Id == id && a.UserId == curUserId)
            .SingleOrDefaultAsync() ??
            throw new ApiError.AtcApplicationNotFound(id);

        var filing = await sheetService.SetSheetFilingAsync(
            ATC_APPLICATION_SHEET_ID,
            application.ApplicationFilingId,
            curUserId,
            req.ApplicationFilingAnswers.ToDictionary(
                answer => answer.Id,
                answer => answer.Answer));

        return new AtcApplicationDto(application);
    }

    protected static AtcApplicationStatusDto ToDto(AtcApplicationStatus status) => status switch
    {
        AtcApplicationStatus.Submitted => AtcApplicationStatusDto.Submitted,
        AtcApplicationStatus.InWaitlist => AtcApplicationStatusDto.InWaitlist,
        AtcApplicationStatus.Approved => AtcApplicationStatusDto.Approved,
        AtcApplicationStatus.Rejected => AtcApplicationStatusDto.Rejected,
        _ => throw new ArgumentOutOfRangeException(nameof(status), "Unknown application status: " + status),
    };

    public record class AtcApplicationSummaryDto(
        Ulid Id,
        Ulid UserId,
        UserDto User,
        DateTimeOffset AppliedAt,
        AtcApplicationStatusDto Status)
    {
        public AtcApplicationSummaryDto(AtcApplication application) : this(
            application.Id,
            application.UserId,
            new(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded")),
            application.AppliedAt,
            ToDto(application.Status))
        { }
    }

    public enum AtcApplicationStatusDto
    {
        Submitted,
        InWaitlist,
        Approved,
        Rejected,
    }

    public record class AtcApplicationDto(
        Ulid Id,
        Ulid UserId,
        UserDto User,
        DateTimeOffset AppliedAt,
        AtcApplicationStatusDto Status,
        IEnumerable<AtcApplicationFieldAnswerDto> ApplicationFilingAnswers,
        IEnumerable<AtcApplicationFieldAnswerDto>? ReviewFilingAnswers = null)
    {
        public AtcApplicationDto(AtcApplication application) : this(
            application.Id,
            application.UserId,
            new(application.User ?? throw new ArgumentNullException(nameof(application), "User must be loaded")),
            application.AppliedAt,
            ToDto(application.Status),
            application.ApplicationFiling?.Answers.Select(answer => new AtcApplicationFieldAnswerDto(answer)) ??
                throw new ArgumentNullException(nameof(application), "ApplicationFiling must be loaded"),
            application.ReviewFiling?.Answers.Select(answer => new AtcApplicationFieldAnswerDto(answer)))
        { }
    }

    public record class AtcApplicationFieldAnswerDto(
        [property: JsonInclude] AtcApplicationSheetFieldDto Field,
        string Answer)
    {
        public AtcApplicationFieldAnswerDto(SheetFilingAnswer answer) : this(
            new AtcApplicationSheetFieldDto(answer.Field ??
                throw new ArgumentNullException(nameof(answer), "Field must be loaded")),
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
            sheet.Fields
                .Where(field => !field.IsDeleted)
                .Select(field => new AtcApplicationSheetFieldDto(field)))
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

    public record class AtcApplicationCreateFieldAnswerDto
    {
        public required string Id { get; set; }
        public required string Answer { get; set; }
    }

    public record class AtcApplicationCreateDto
    {
        public required IEnumerable<AtcApplicationCreateFieldAnswerDto> ApplicationFilingAnswers { get; set; }
    }
}
