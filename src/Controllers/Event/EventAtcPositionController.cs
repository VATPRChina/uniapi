using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

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
            .SingleOrDefaultAsync(x => x.Id == controllerId && x.EventId == eventId)
            ?? throw new ApiError.EventAtcPositionNotFound(eventId, controllerId);
        return slot;
    }

    [HttpGet]
    public async Task<IEnumerable<EventAtcPositionDto>> GetEventAtcPositionsAsync(Ulid eventId)
    {
        var positions = await DbContext.EventAtcPosition
            .Where(x => x.EventId == eventId)
            .Select(x => new EventAtcPositionDto(x))
            .ToListAsync();
        return positions;
    }

    [HttpPost]
    [Authorize(Roles = EDIT_ROLES)]
    public async Task<EventAtcPositionDto> CreateEventAtcPositionAsync(Ulid eventId, EventAtcPositionDto dto)
    {
        if (positionKindService.GetById(dto.PositionKindId) == null)
        {
            throw new ApiError.InvalidAtcPositionKind(dto.PositionKindId);
        }
        var position = new EventAtcPosition
        {
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
        return new EventAtcPositionDto(position);
    }

    [HttpPut("{positionId}")]
    [Authorize(Roles = EDIT_ROLES)]
    public async Task<EventAtcPositionDto> UpdateEventAtcPositionAsync(Ulid eventId, Ulid positionId, CreateEventAtcPositionDto dto)
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
        return new EventAtcPositionDto(position);
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
    public async Task<EventAtcPositionBookingDto> BookEventAtcPositionAsync(Ulid eventId, Ulid positionId)
    {
        var position = await LoadAsync(eventId, positionId);

        var user = await userAccessor.GetUser();
        var userPositionPermission = await DbContext.UserAtcPermission
            .SingleOrDefaultAsync(x => x.UserId == user.Id && x.PositionKindId == position.PositionKindId)
            ?? throw new ApiError.InsufficientAtcPermission(position.PositionKindId, null, position.MinimumControllerState);
        if (!positionStatusService.IsStatusSatifyMinimum(userPositionPermission, position.MinimumControllerState))
        {
            throw new ApiError.InsufficientAtcPermission(position.PositionKindId, userPositionPermission.State, position.MinimumControllerState);
        }

        if (position.Booking != null)
        {
            throw new ApiError.EventPositionBooked(eventId, positionId);
        }

        position.Booking = new EventAtcPositionBooking
        {
            UserId = user.Id,
            CreatedAt = DateTimeOffset.UtcNow,
        };

        await DbContext.SaveChangesAsync();
        return new EventAtcPositionBookingDto(position.Booking);
    }

    [HttpDelete("{positionId}/booking")]
    [Authorize(Roles = UserRoles.Controller)]
    public async Task<EventAtcPositionBookingDto> CancelEventAtcPositionBookingAsync(Ulid eventId, Ulid positionId)
    {
        var position = await LoadAsync(eventId, positionId);
        var user = await userAccessor.GetUser();

        if (position.Booking == null || position.Booking.UserId != user.Id)
        {
            throw new ApiError.EventPositionNotBooked(eventId, positionId);
        }

        if (position.Booking.UserId != user.Id)
        {
            throw new ApiError.EventPositionBookedByAnotherUser(eventId, positionId);
        }

        DbContext.EventAtcPositionBooking.Remove(position.Booking);
        await DbContext.SaveChangesAsync();

        return new EventAtcPositionBookingDto(position.Booking);
    }

    public record EventAtcPositionDto(
        string Callsign,
        DateTimeOffset StartAt,
        DateTimeOffset EndAt,
        string? Remarks,
        string PositionKindId,
        UserControllerState MinimumControllerState,
        EventAtcPositionBookingDto? Booking
    )
    {
        public EventAtcPositionDto(EventAtcPosition position) : this(
            position.Callsign,
            position.StartAt,
            position.EndAt,
            position.Remarks,
            position.PositionKindId,
            position.MinimumControllerState,
            position.Booking != null ? new EventAtcPositionBookingDto(position.Booking) : null)
        {
        }
    }

    public record CreateEventAtcPositionDto
    {
        public required string Callsign { get; set; }
        public required DateTimeOffset StartAt { get; set; }
        public required DateTimeOffset EndAt { get; set; }
        public string? Remarks { get; set; }
        public required string PositionKindId { get; set; }
        public required UserControllerState MinimumControllerState { get; set; }
    }

    public record EventAtcPositionBookingDto(
        Ulid UserId,
        DateTimeOffset BookedAt
    )
    {
        public EventAtcPositionBookingDto(EventAtcPositionBooking booking) : this(
            booking.UserId,
            booking.CreatedAt)
        {
        }
    }
}
