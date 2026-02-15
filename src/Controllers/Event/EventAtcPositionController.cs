using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eventId}/controllers")]
public class EventAtcPositionController(
    Database DbContext,
    IUserAccessor userAccessor,
    AtcPositionKindService positionKindService,
    AtcPositionStatusService positionStatusService) : ControllerBase
{
    private const string EDIT_ROLES = $"{UserRoles.EventCoordinator},{UserRoles.ControllerTrainingDirectorAssistant},{UserRoles.OperationDirectorAssistant}";

    protected async Task<EventAtcPosition> LoadAsync(Ulid eventId, Ulid controllerId)
    {
        var slot = await DbContext.EventAtcPosition
            .Include(x => x.Event)
            .Include(x => x.Booking)
                .ThenInclude(x => x!.User)
            .Include(x => x.Booking)
                .ThenInclude(x => x!.AtcBooking)
            .SingleOrDefaultAsync(x => x.Id == controllerId && x.EventId == eventId)
            ?? throw new ApiError.EventAtcPositionNotFound(eventId, controllerId);
        return slot;
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventAtcPositionDto>> GetEventAtcPositionsAsync(Ulid eventId)
    {
        var positions = await DbContext.EventAtcPosition
            .Where(x => x.EventId == eventId)
            .Include(x => x.Event)
            .Include(x => x.Booking)
                .ThenInclude(x => x!.User)
            .Select(x => EventAtcPositionDto.From(x))
            .ToListAsync();
        return positions;
    }

    [HttpPost]
    [Authorize(Roles = EDIT_ROLES)]
    public async Task<EventAtcPositionDto> CreateEventAtcPositionAsync(Ulid eventId, EventAtcPositionSaveRequest dto)
    {
        if (positionKindService.GetById(dto.PositionKindId) == null)
        {
            throw new ApiError.InvalidAtcPositionKind(dto.PositionKindId);
        }
        var position = new EventAtcPosition
        {
            Id = Ulid.NewUlid(),
            EventId = eventId,
            Callsign = dto.Callsign,
            StartAt = dto.StartAt,
            EndAt = dto.EndAt,
            Remarks = dto.Remarks,
            PositionKindId = dto.PositionKindId,
            MinimumControllerState = dto.MinimumControllerState,
        };
        DbContext.EventAtcPosition.Add(position);
        await DbContext.SaveChangesAsync();
        await DbContext.Entry(position).Reference(x => x.Event).LoadAsync();
        await DbContext.Entry(position).Reference(x => x.Booking).LoadAsync();
        if (position.Booking != null)
        {
            await DbContext.Entry(position.Booking).Reference(x => x.User).LoadAsync();
        }
        return EventAtcPositionDto.From(position);
    }

    [HttpPut("{positionId}")]
    [Authorize(Roles = EDIT_ROLES)]
    public async Task<EventAtcPositionDto> UpdateEventAtcPositionAsync(Ulid eventId, Ulid positionId, EventAtcPositionSaveRequest dto)
    {
        var position = await LoadAsync(eventId, positionId);

        if (positionKindService.GetById(dto.PositionKindId) == null)
        {
            throw new ApiError.InvalidAtcPositionKind(dto.PositionKindId);
        }

        position.Callsign = dto.Callsign;
        position.StartAt = dto.StartAt;
        position.EndAt = dto.EndAt;
        position.Remarks = dto.Remarks;
        position.PositionKindId = dto.PositionKindId;
        position.MinimumControllerState = dto.MinimumControllerState;

        await DbContext.SaveChangesAsync();
        await DbContext.Entry(position).Reference(x => x.Event).LoadAsync();
        await DbContext.Entry(position).Reference(x => x.Booking).LoadAsync();
        if (position.Booking != null)
        {
            await DbContext.Entry(position.Booking).Reference(x => x.User).LoadAsync();
        }
        return EventAtcPositionDto.From(position);
    }

    [HttpDelete("{positionId}")]
    [Authorize(Roles = EDIT_ROLES)]
    public async Task DeleteEventAtcPositionAsync(Ulid eventId, Ulid positionId)
    {
        var position = await LoadAsync(eventId, positionId);
        DbContext.EventAtcPosition.Remove(position);
        await DbContext.SaveChangesAsync();
    }

    [HttpPut("{positionId}/booking")]
    [Authorize(Roles = UserRoles.Controller)]
    public async Task<EventAtcPositionBookingDto> BookEventAtcPositionAsync(Ulid eventId, Ulid positionId, EventAtcPositionBookRequest req)
    {
        var position = await LoadAsync(eventId, positionId);

        if (req.UserId != null)
        {
            await userAccessor.EnsureCurrentUserAnyRoleOf(
                UserRoles.EventCoordinator,
                UserRoles.ControllerTrainingDirectorAssistant,
                UserRoles.ControllerTrainingMentor);
        }

        var userId = req.UserId ?? userAccessor.GetUserId();
        var userPositionPermission = await DbContext.UserAtcPermission
            .SingleOrDefaultAsync(x => x.UserId == userId && x.PositionKindId == position.PositionKindId)
            ?? throw new ApiError.InsufficientAtcPermission(position.PositionKindId, null, position.MinimumControllerState);
        if (!positionStatusService.IsStatusSatifyMinimum(userPositionPermission, position.MinimumControllerState))
        {
            throw new ApiError.InsufficientAtcPermission(position.PositionKindId, userPositionPermission.State, position.MinimumControllerState);
        }

        if (position.Booking != null)
        {
            throw new ApiError.EventPositionBooked(eventId, positionId);
        }

        if (position.Event!.IsInAtcBookingPeriod == false && req.UserId == null)
        {
            throw new ApiError.EventNotInBookingTime(eventId);
        }

        position.Booking = new EventAtcPositionBooking
        {
            EventAtcPositionId = position.Id,
            UserId = userId,
            AtcBookingId = Ulid.NewUlid(),
            CreatedAt = DateTimeOffset.UtcNow,
        };

        var atcBooking = new AtcBooking
        {
            Id = position.Booking.AtcBookingId,
            UserId = userId,
            Callsign = position.Callsign,
            BookedAt = DateTimeOffset.UtcNow,
            StartAt = position.StartAt,
            EndAt = position.EndAt,
        };
        DbContext.AtcBooking.Add(atcBooking);

        await DbContext.SaveChangesAsync();

        await DbContext.Entry(position.Booking).Reference(x => x.User).LoadAsync();
        return EventAtcPositionBookingDto.From(position.Booking);
    }

    [HttpDelete("{positionId}/booking")]
    [Authorize(Roles = UserRoles.Controller)]
    public async Task<EventAtcPositionBookingDto> CancelEventAtcPositionBookingAsync(Ulid eventId, Ulid positionId)
    {
        var position = await LoadAsync(eventId, positionId);
        var user = await userAccessor.GetUser();

        if (position.Booking == null)
        {
            throw new ApiError.EventPositionNotBooked(eventId, positionId);
        }

        var isAdmin = await userAccessor.HasCurrentUserAnyRoleOf(
            UserRoles.EventCoordinator,
            UserRoles.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingMentor);
        if (position.Booking.UserId != user.Id && !isAdmin)
        {
            throw new ApiError.EventPositionBookedByAnotherUser(eventId, positionId);
        }

        if (position.Booking.AtcBooking != null)
        {
            DbContext.AtcBooking.Remove(position.Booking.AtcBooking);
        }

        DbContext.EventAtcPositionBooking.Remove(position.Booking);
        await DbContext.SaveChangesAsync();

        return EventAtcPositionBookingDto.From(position.Booking);
    }
}
