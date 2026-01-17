using Discord.Commands;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using Org.BouncyCastle.Ocsp;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/trainings")]
public class TrainingController(
    Database database,
    SheetService sheetService,
    IUserAccessor userAccessor
) : Controller
{
    public const string RECORD_SHEEET_ID = "training-record";

    [HttpGet("active")]
    public async Task<IEnumerable<TrainingDto>> List()
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingMentor);

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
    public async Task<IEnumerable<TrainingDto>> ListFinished()
    {
        var isAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingMentor);

        var trainings = await database.Training
            .Where(t => t.RecordSheetFilingId != null)
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

    [HttpPost]
    [RequireRole(UserRoles.ControllerTrainingMentor)]
    public async Task<TrainingDto> Create([FromBody] TrainingSaveRequest request)
    {
        var isSuperAdmin = await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingDirectorAssistant);

        if (request.TrainerId != userAccessor.GetUserId() && !isSuperAdmin)
        {
            throw new ApiError.CannotCreateTrainingForOtherTrainers();
        }

        var training = new Training
        {
            Id = Ulid.NewUlid(),
            Name = request.Name,
            TrainerId = request.TrainerId,
            TraineeId = request.TraineeId,
            StartAt = request.StartAt,
            EndAt = request.EndAt,
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow,
        };

        database.Training.Add(training);
        await database.SaveChangesAsync();

        var createdTraining = await FindById(training.Id);

        return TrainingDto.From(createdTraining);
    }

    [HttpPut("{id}")]
    [RequireRole(UserRoles.ControllerTrainingMentor)]
    public async Task<TrainingDto> Update(Ulid id, [FromBody] TrainingSaveRequest request)
    {
        var training = await FindById(id);

        if (training == null)
        {
            throw new ApiError.NotFound(nameof(database.Training), id);
        }

        await ValidateOwnership(training, requireTrainer: true);

        if (request.TrainerId != training.TrainerId ||
           request.TraineeId != training.TraineeId)
        {
            throw new ApiError.CannotUpdateTrainingTrainerTrainee();
        }

        training.Name = request.Name;
        training.StartAt = request.StartAt;
        training.EndAt = request.EndAt;
        training.UpdatedAt = DateTimeOffset.UtcNow;

        await database.SaveChangesAsync();

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
    [RequireRole(UserRoles.ControllerTrainingMentor)]
    [ApiError.Has<ApiError.NotFound>]
    public async Task<TrainingDto> SetRecordSheet(Ulid id, [FromBody]
        TrainingRecordRequest request)
    {
        var training = await FindById(id);

        await ValidateOwnership(training, requireTrainer: true);

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

    protected async Task ValidateOwnership(Training training, bool requireTrainer = false)
    {
        if (training.TrainerId == userAccessor.GetUserId()) return;
        if (training.TraineeId == userAccessor.GetUserId() && !requireTrainer) return;
        if (await userAccessor.HasCurrentUserRole(UserRoles.ControllerTrainingMentor)) return;

        throw new ApiError.NotOwned(nameof(database.Training), training.Id, userAccessor.GetUserId());
    }

    protected async Task<Training> FindById(Ulid id)
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

        return training;
    }
}
