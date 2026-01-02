using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/trainings")]
public class TrainingController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor
) : Controller
{
    protected const int MAX_TRAININGS_PER_PAGE = 50;
    public const string RECORD_SHEEET_ID = "training-record";

    [HttpGet("active")]
    public async Task<IEnumerable<TrainingDto>> List()
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);

        var trainings = await database.Training
            .Where(t => t.RecordSheetFilingId == null)
            .Where(t => isAdmin ||
                t.TrainerId == userAccessor.GetUserId() ||
                t.TraineeId == userAccessor.GetUserId())
            .OrderByDescending(t => t.CreatedAt)
            .Include(t => t.Trainer)
            .Include(t => t.Trainee)
            .Include(t => t.RecordSheetFiling)
                .ThenInclude(s => s!.Answers!)
                    .ThenInclude(a => a.Field)
            .Select(t => TrainingDto.From(t))
            .ToListAsync();

        return trainings;
    }

    [HttpGet("finished")]
    public async Task<IEnumerable<TrainingDto>> ListFinished(DateTimeOffset? until = null)
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);

        var trainings = await database.Training
            .Where(t => t.RecordSheetFilingId != null
                && (until == null || t.CreatedAt <= until))
            .Where(t => isAdmin ||
                t.TrainerId == userAccessor.GetUserId() ||
                t.TraineeId == userAccessor.GetUserId())
            .OrderByDescending(t => t.CreatedAt)
            .Include(t => t.Trainer)
            .Include(t => t.Trainee)
            .Include(t => t.RecordSheetFiling)
                .ThenInclude(s => s!.Answers!)
                    .ThenInclude(a => a.Field)
            .Take(MAX_TRAININGS_PER_PAGE)
            .Select(t => TrainingDto.From(t))
            .ToListAsync();

        return trainings;
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.NotFound>]
    public async Task<TrainingDto> Get(Ulid id)
    {
        var training = await database.Training
            .Where(t => t.Id == id)
            .Include(t => t.Trainer)
            .Include(t => t.Trainee)
            .Include(t => t.RecordSheetFiling)
                .ThenInclude(s => s!.Answers!)
                    .ThenInclude(a => a.Field)
            .FirstOrDefaultAsync();

        if (training == null)
        {
            throw new ApiError.NotFound(nameof(database.Training), id);
        }

        await ValidateOwnership(training);

        return TrainingDto.From(training);
    }

    [HttpGet("record-sheet")]
    public async Task<SheetDto> GetSheet()
    {
        await sheetService.EnsureSheetAsync(RECORD_SHEEET_ID, "Training Record Sheet");
        var sheet = await sheetService.GetSheetByIdAsync(RECORD_SHEEET_ID)
            ?? throw new InvalidOperationException("Training record sheet is not configured.");
        return SheetDto.From(sheet);
    }

    [HttpPut("{id}/record")]
    [ApiError.Has<ApiError.NotFound>]
    public async Task<TrainingDto> SetRecordSheet(Ulid id, [FromBody]
        TrainingRecordRequest request)
    {
        var training = await database.Training
            .Include(t => t.Trainer)
            .Include(t => t.Trainee)
            .Include(t => t.RecordSheetFiling)
                .ThenInclude(s => s!.Answers!)
                    .ThenInclude(a => a.Field)
            .Where(t => t.Id == id)
            .SingleOrDefaultAsync();
        if (training == null)
        {
            throw new ApiError.NotFound(nameof(database.Training), id);
        }

        await ValidateOwnership(training);

        var recordFiling = await sheetService.SetSheetFilingAsync(
            RECORD_SHEEET_ID,
            training.RecordSheetFiling?.Id,
            training.TrainerId,
            request.RequestAnswers.ToDictionary(kv => kv.Id, kv => kv.Answer),
            default);

        training.RecordSheetFilingId = recordFiling.Id;
        training.UpdatedAt = DateTimeOffset.UtcNow;
        await database.SaveChangesAsync();

        return TrainingDto.From(training);
    }

    protected async Task ValidateOwnership(Training training)
    {
        if (training.TrainerId == userAccessor.GetUserId()) return;
        if (training.TraineeId == userAccessor.GetUserId()) return;
        if (await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant)) return;

        throw new ApiError.NotOwned(nameof(database.Training), training.Id, userAccessor.GetUserId());
    }
}
