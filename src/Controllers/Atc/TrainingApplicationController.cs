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
[Route("api/atc/trainings/applications")]
public class TrainingApplicationController(
    Database database,
    IUserAccessor userAccessor,
    ISmtpEmailAdapter emailAdapter
) : Controller
{
    protected const int MAX_TRAININGS_PER_PAGE = 50;

    protected async Task<bool> IsAdmin()
    {
        return await userAccessor.HasCurrentUserAnyRoleOf(
            UserRoles.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingMentor);
    }

    [HttpGet]
    public async Task<IEnumerable<TrainingApplicationDto>> List()
    {
        var isAdmin = await IsAdmin();

        var trainings = await database.TrainingApplication
            .Where(t => isAdmin || t.TraineeId == userAccessor.GetUserId())
            .OrderByDescending(t => t.CreatedAt)
            .Include(t => t.Trainee)
            .Include(t => t.Slots)
            .Select(t => TrainingApplicationDto.From(t))
            .ToListAsync();

        return trainings;
    }

    protected async Task<TrainingApplication> FindById(Ulid id)
    {
        var isAdmin = await IsAdmin();

        var training = await database.TrainingApplication
            .Where(t => t.Id == id && (isAdmin || t.TraineeId == userAccessor.GetUserId()))
            .Include(t => t.Trainee)
            .Include(t => t.Slots)
            .SingleOrDefaultAsync()
            ?? throw new ApiError.NotFound(nameof(database.TrainingApplication), id);

        return training;
    }

    [HttpGet("{id}")]
    public async Task<TrainingApplicationDto> GetById(Ulid id)
    {
        var training = await FindById(id);

        return TrainingApplicationDto.From(training);
    }

    [HttpDelete("{id}")]
    public async Task<TrainingApplicationDto> Delete(Ulid id)
    {
        var trainingApplication = await FindById(id);

        trainingApplication.DeletedAt = DateTimeOffset.UtcNow;

        await database.SaveChangesAsync();

        return TrainingApplicationDto.From(trainingApplication);
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
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow
        };

        database.TrainingApplication.Add(trainingApplication);

        var slots = dto.Slots.Select(slotDto => new TrainingApplicationSlot
        {
            Id = Ulid.NewUlid(),
            ApplicationId = trainingApplication.Id,
            StartAt = slotDto.StartAt,
            EndAt = slotDto.EndAt
        });
        database.TrainingApplicationSlot.AddRange(slots);

        await database.SaveChangesAsync();

        return TrainingApplicationDto.From(trainingApplication);
    }

    [HttpPut("{id}")]
    public async Task<TrainingApplicationDto> Update(Ulid id, [FromBody] TrainingApplicationCreateRequest dto)
    {
        var trainingApplication = await FindById(id);

        trainingApplication.Name = dto.Name;
        trainingApplication.UpdatedAt = DateTimeOffset.UtcNow;

        var existingSlots = await database.TrainingApplicationSlot
            .Where(s => s.ApplicationId == trainingApplication.Id)
            .ToListAsync();

        database.TrainingApplicationSlot.RemoveRange(
            existingSlots.Where(s => !dto.Slots.Any(dtoSlot => dtoSlot.StartAt == s.StartAt && dtoSlot.EndAt == s.EndAt)));

        var slots = dto.Slots
            .Where(dtoSlot => !existingSlots.Any(s => s.StartAt == dtoSlot.StartAt && s.EndAt == dtoSlot.EndAt))
            .Select(slotDto => new TrainingApplicationSlot
            {
                Id = Ulid.NewUlid(),
                ApplicationId = trainingApplication.Id,
                StartAt = slotDto.StartAt,
                EndAt = slotDto.EndAt
            });
        database.TrainingApplicationSlot.AddRange(slots);

        await database.SaveChangesAsync();

        return TrainingApplicationDto.From(trainingApplication);
    }

    [HttpGet("{id}/responses")]
    public async Task<IEnumerable<TrainingApplicationResponseDto>> GetResponses(Ulid id)
    {
        var application = await FindById(id);

        var responses = await database.TrainingApplicationResponse
            .Where(r => r.ApplicationId == id)
            .Include(r => r.Application)
                .ThenInclude(a => a!.Trainee)
            .Include(r => r.Trainer)
            .OrderByDescending(r => r.CreatedAt)
            .Select(r => TrainingApplicationResponseDto.From(r))
            .ToListAsync();

        return responses;
    }

    [HttpPut("{id}/response")]
    [Authorize(Roles = UserRoles.ControllerTrainingMentor)]
    public async Task<TrainingApplicationResponseDto> RespondToApplication(
        Ulid id,
        [FromBody] TrainingApplicationResponseRequest req)
    {
        var application = await database.TrainingApplication
            .Include(a => a.Trainee)
            .Include(a => a.Train)
            .FirstOrDefaultAsync(a => a.Id == id)
            ?? throw new ApiError.NotFound(nameof(database.TrainingApplication), id);

        if (application.TrainId != null)
        {
            throw new ApiError.TrainingApplicationAlreadyAccepted(id);
        }

        if (application.DeletedAt != null)
        {
            throw new ApiError.NotFound(nameof(TrainingApplication), id);
        }

        TrainingApplicationSlot? slot = null;
        if (req.SlotId != null)
        {
            slot = await database.TrainingApplicationSlot
                .FirstOrDefaultAsync(s => s.Id == req.SlotId && s.ApplicationId == application.Id)
                ?? throw new ApiError.NotFound(nameof(database.TrainingApplicationSlot), req.SlotId.Value);
        }

        var response = new TrainingApplicationResponse
        {
            Id = Ulid.NewUlid(),
            ApplicationId = application.Id,
            TrainerId = userAccessor.GetUserId(),
            SlotId = req.SlotId,
            Comment = req.Comment,
            CreatedAt = DateTimeOffset.UtcNow,
            UpdatedAt = DateTimeOffset.UtcNow,
        };

        database.TrainingApplicationResponse.Add(response);

        if (slot != null)
        {
            var training = new Training
            {
                Id = Ulid.NewUlid(),
                Name = application.Name,
                TrainerId = response.TrainerId,
                TraineeId = application.TraineeId,
                StartAt = slot.StartAt,
                EndAt = slot.EndAt,
                CreatedAt = DateTimeOffset.UtcNow,
                UpdatedAt = DateTimeOffset.UtcNow
            };
            database.Training.Add(training);

            application.TrainId = training.Id;
            application.UpdatedAt = DateTimeOffset.UtcNow;
        }

        await database.SaveChangesAsync();

        if (application.Trainee!.Email != null)
        {
            await database.Entry(response).Reference(u => u.Trainer).LoadAsync();
            var email = new TrainingApplicationResponseEmail(application, response);
            await emailAdapter.SendEmailAsync(application.Trainee.Email, email, default);
        }

        return TrainingApplicationResponseDto.From(response);
    }
}
