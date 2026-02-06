using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Event;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots")]
public class EventSlotController(
    Database DbContext,
    IUserAccessor userAccessor) : ControllerBase
{
    protected async Task<EventSlot> LoadAsync(Ulid eid, Ulid sid)
    {
        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
                .ThenInclude(b => b!.User)
            .SingleOrDefaultAsync(x => x.Id == sid && x.EventAirspace.EventId == eid)
            ?? throw new ApiError.EventSlotNotFound(eid, sid);
        return slot;
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventSlotDto>> List(Ulid eid)
    {
        var eventt = await DbContext.Event
            .Include(x => x.Airspaces!)
            .ThenInclude(x => x.Slots)
                .ThenInclude(x => x.Booking!)
                    .ThenInclude(b => b.User)
            .SingleOrDefaultAsync(x => x.Id == eid)
            ?? throw new ApiError.EventNotFound(eid);
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return eventt.Airspaces!
            .SelectMany(x => x.Slots)
            .OrderBy(x => x.EnterAt)
            .ThenBy(x => x.LeaveAt)
            .Select(x => EventSlotDto.From(x, includeBookingUser));
    }

    [HttpGet("bookings.csv")]
    [Produces("text/csv")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<IActionResult> Export(Ulid eid)
    {
        var slots = await DbContext.EventBooking
            .Include(x => x.User)
            .Include(x => x.EventSlot)
                .ThenInclude(x => x.EventAirspace)
            .Where(x => x.EventSlot.EventAirspace.EventId == eid)
            .OrderBy(x => x.EventSlot.EnterAt)
            .Select(x => $"{x.User.Cid},{x.EventSlot.EnterAt:HHmm}")
            .ToArrayAsync();
        return File(System.Text.Encoding.UTF8.GetBytes(string.Join("\n", slots)), "text/csv", "bookings.csv");
    }

    [HttpGet("{sid}")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventSlotDto> Get(Ulid eid, Ulid sid)
    {
        var slot = await LoadAsync(eid, sid);
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventSlotDto.From(slot, includeBookingUser);
    }

    [HttpGet("mine")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventSlotNotFound>]
    public async Task<EventSlotDto> GetMine(Ulid eid)
    {
        var user = await userAccessor.GetUser();

        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
            .FirstOrDefaultAsync(f => f.EventAirspace!.Id == eid && f.Booking!.UserId == user.Id)
            ?? throw new ApiError.EventSlotNotFoundForUser(eid, user.Id);
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventSlotDto.From(slot, includeBookingUser);
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    [ApiError.Has<ApiError.EventAirspaceNotFound>]
    public async Task<EventSlotDto> Create(Ulid eid, EventSlotSaveRequest dto)
    {
        var airspace = await DbContext.EventAirspace.FindAsync(dto.AirspaceId)
            ?? throw new ApiError.EventAirspaceNotFound(eid, dto.AirspaceId);
        var slot = new EventSlot()
        {
            EventAirspace = airspace,
            EnterAt = dto.EnterAt.ToUniversalTime(),
            LeaveAt = dto.LeaveAt?.ToUniversalTime(),
            Callsign = dto.Callsign,
            AircraftTypeIcao = dto.AircraftTypeIcao,
        };
        DbContext.EventSlot.Add(slot);
        await DbContext.SaveChangesAsync();
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventSlotDto.From(slot, includeBookingUser);
    }

    [HttpPut("{sid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Update(Ulid eid, Ulid sid, EventSlotSaveRequest dto)
    {
        var slot = await LoadAsync(eid, sid);
        slot.EnterAt = dto.EnterAt;
        slot.LeaveAt = dto.LeaveAt;
        slot.Callsign = dto.Callsign;
        slot.AircraftTypeIcao = dto.AircraftTypeIcao;
        await DbContext.SaveChangesAsync();
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventSlotDto.From(slot, includeBookingUser);
    }

    [HttpDelete("{sid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Delete(Ulid eid, Ulid sid)
    {
        var slot = await LoadAsync(eid, sid);
        DbContext.EventSlot.Remove(slot);
        await DbContext.SaveChangesAsync();
        var includeBookingUser = await userAccessor.HasCurrentUserAnyRoleOf(UserRoles.EventCoordinator, UserRoles.Controller);
        return EventSlotDto.From(slot, includeBookingUser);
    }
}
