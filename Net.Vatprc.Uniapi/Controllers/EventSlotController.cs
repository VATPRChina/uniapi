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
        public Ulid AirspaceId { get; set; }
        public EventAirspaceController.EventAirspaceDto Airspace { get; set; }
        public DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset? LeaveAt { get; set; }
        public DateTimeOffset CreatedAt { get; set; }
        public DateTimeOffset UpdatedAt { get; set; }
        public EventSlotBookingController.EventBookingDto? Booking { get; set; }

        public EventSlotDto(EventSlot slot)
        {
            Id = slot.Id;
            EventId = slot.EventAirspace.EventId;
            AirspaceId = slot.EventAirspaceId;
            Airspace = new(slot.EventAirspace);
            EnterAt = slot.EnterAt;
            CreatedAt = slot.CreatedAt;
            UpdatedAt = slot.UpdatedAt;
            LeaveAt = slot.LeaveAt;
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
            .ThenInclude(x => x.Booking)
            .SingleOrDefaultAsync(x => x.Id == eid)
            ?? throw new ApiError.EventNotFound(eid);
        return eventt.Airspaces.SelectMany(x => x.Slots).OrderBy(x => x.EnterAt).Select(x => new EventSlotDto(x)).ToArray();
    }

    [HttpGet("bookings.csv")]
    [Produces("text/csv")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
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
        return new(slot);
    }

    public record CreateEventSlotDto
    {
        public required Ulid AirspaceId { get; set; }
        public required DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset? LeaveAt { get; set; }
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
            EnterAt = dto.EnterAt.ToUniversalTime(),
            LeaveAt = dto.LeaveAt?.ToUniversalTime(),
        };
        DbContext.EventSlot.Add(slot);
        await DbContext.SaveChangesAsync();
        return new(slot);
    }

    public record UpdateEventSlotDto
    {
        public required DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset? LeaveAt { get; set; }
    }

    [HttpPut("{sid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Update(Ulid eid, Ulid sid, UpdateEventSlotDto dto)
    {
        var slot = await LoadAsync(eid, sid);
        slot.EnterAt = dto.EnterAt;
        slot.LeaveAt = dto.LeaveAt;
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
