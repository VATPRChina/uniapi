using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/atc/trainings/applications")]
public class TrainingApplicationController(
    Database database,
    IUserAccessor userAccessor
) : Controller
{
    protected const int MAX_TRAININGS_PER_PAGE = 50;

    [HttpGet]
    public async Task<IEnumerable<TrainingApplicationDto>> List()
    {
        var isAdmin = await userAccessor.HasCurrentUserAnyRoleOf(
            UserRoles.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingMentor);

        var trainings = await database.TrainingApplication
            .Where(t => isAdmin || t.TraineeId == userAccessor.GetUserId())
            .OrderByDescending(t => t.CreatedAt)
            .Select(t => TrainingApplicationDto.From(t))
            .ToListAsync();

        return trainings;
    }

    [HttpGet("{id}")]
    public async Task<TrainingApplicationDto> GetById(Ulid id)
    {
        var isAdmin = await userAccessor.HasCurrentUserAnyRoleOf(
            UserRoles.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingMentor);

        var training = await database.TrainingApplication
            .Where(t => t.Id == id && (isAdmin || t.TraineeId == userAccessor.GetUserId()))
            .Include(t => t.Trainee)
            .Include(t => t.Train)
            .SingleOrDefaultAsync()
            ?? throw new ApiError.NotFound(nameof(database.TrainingApplication), id);

        return TrainingApplicationDto.From(training);
    }

    [HttpPost]
    public async Task<TrainingApplicationDto> Create([FromBody] TrainingApplicationCreateRequest dto)
    {
        var userId = userAccessor.GetUserId();

        var atcPermissions = await database.UserAtcPermission
            .Where(p => p.UserId == userId)
            .ToListAsync();
        if (!atcPermissions.Any(p => p.CanRequestMentorSession))
        {
            throw new ApiError.Forbidden(["controller/*/mentee"]);
        }

        var trainingApplication = new TrainingApplication
        {
            Id = Ulid.NewUlid(),
            TraineeId = userId,
            Name = dto.Name,
            StartAt = dto.StartAt,
            EndAt = dto.EndAt,
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow
        };

        database.TrainingApplication.Add(trainingApplication);
        await database.SaveChangesAsync();

        return TrainingApplicationDto.From(trainingApplication);
    }

    [HttpPut("{id}/response")]
    [Authorize(Roles = UserRoles.ControllerTrainingMentor)]
    public async Task<TrainingApplicationResponseDto> RespondToApplication(
        Ulid id,
        [FromBody] TrainingApplicationResponseRequest req)
    {
        var application = await database.TrainingApplication
            .Include(a => a.Trainee)
            .FirstOrDefaultAsync(a => a.Id == id)
            ?? throw new ApiError.NotFound(nameof(database.TrainingApplication), id);

        var response = new TrainingApplicationResponse
        {
            Id = Ulid.NewUlid(),
            ApplicationId = application.Id,
            TrainerId = userAccessor.GetUserId(),
            IsAccepted = req.IsAccepted,
            Comment = req.Comment,
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow
        };

        database.TrainingApplicationResponse.Add(response);

        if (req.IsAccepted)
        {
            var training = new Training
            {
                Id = Ulid.NewUlid(),
                Name = application.Name,
                TrainerId = response.TrainerId,
                TraineeId = application.TraineeId,
                StartAt = application.StartAt,
                EndAt = application.EndAt,
                CreatedAt = DateTimeOffset.UtcNow,
                UpdatedAt = DateTimeOffset.UtcNow
            };
            database.Training.Add(training);

            application.TrainId = training.Id;
            application.UpdatedAt = DateTimeOffset.UtcNow;
        }

        await database.SaveChangesAsync();

        return TrainingApplicationResponseDto.From(response);
    }
}
