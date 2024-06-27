using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots")]
public class EventSlotController(VATPRCContext DbContext) : ControllerBase
{
    protected async Task<EventSlot> LoadAsync(Ulid eid, Ulid sid)
    {
        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
            .SingleOrDefaultAsync(x => x.Id == sid && x.EventAirspace.EventId == eid)
            ?? throw new ApiError.EventSlotNotFound(eid, sid);
        return slot;
    }

    public record EventSlotDto
    {
        public Ulid Id { get; set; }
        public Ulid EventId { get; set; }
        public Ulid EventAirspaceId { get; set; }
        public DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset CreatedAt { get; set; }
        public DateTimeOffset UpdatedAt { get; set; }
        public EventSlotBookingController.EventBookingDto? Booking { get; set; }


        public EventSlotDto(EventSlot slot)
        {
            Id = slot.Id;
            EventId = slot.EventAirspace.EventId;
            EventAirspaceId = slot.EventAirspaceId;
            EnterAt = slot.EnterAt;
            CreatedAt = slot.CreatedAt;
            UpdatedAt = slot.UpdatedAt;
            if (slot.Booking != null) Booking = new(slot.Booking);
        }
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventSlotDto>> List(Ulid eid)
    {
        var eventt = await DbContext.Event
            .Include(x => x.Airspaces)
            .ThenInclude(x => x.Slots)
            .SingleOrDefaultAsync(x => x.Id == eid)
            ?? throw new ApiError.EventNotFound(eid);
        return eventt.Airspaces.SelectMany(x => x.Slots).Select(x => new EventSlotDto(x)).ToArray();
    }

    [HttpGet("{sid}")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventSlotDto> Get(Ulid eid, Ulid sid)
    {
        var slot = await LoadAsync(eid, sid);
        return new(slot);
    }

    public record CreateEventSlotDto
    {
        public required Ulid AirspaceId { get; set; }
        public required DateTimeOffset EnterAt { get; set; }
    }

    [HttpPost]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Create(Ulid eid, CreateEventSlotDto dto)
    {
        var airspace = await DbContext.EventAirspace.FindAsync(dto.AirspaceId)
            ?? throw new ApiError.EventAirspaceNotFound(eid, dto.AirspaceId);
        var slot = new EventSlot()
        {
            EventAirspace = airspace,
            EnterAt = dto.EnterAt,
        };
        DbContext.EventSlot.Add(slot);
        await DbContext.SaveChangesAsync();
        return new(slot);
    }

    public record UpdateEventSlotDto
    {
        public required DateTimeOffset EnterAt { get; set; }
    }

    [HttpPut("{sid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Update(Ulid eid, Ulid sid, UpdateEventSlotDto dto)
    {
        var slot = await LoadAsync(eid, sid);
        slot.EnterAt = dto.EnterAt;
        await DbContext.SaveChangesAsync();
        return new(slot);
    }

    [HttpDelete("{sid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Delete(Ulid eid, Ulid sid)
    {
        var slot = await LoadAsync(eid, sid);
        DbContext.EventSlot.Remove(slot);
        await DbContext.SaveChangesAsync();
        return new(slot);
    }
}
