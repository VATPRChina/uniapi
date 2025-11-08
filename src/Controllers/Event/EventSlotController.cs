using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/slots")]
public class EventSlotController(Database DbContext) : ControllerBase
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
        public string? Callsign { get; set; }
        public string? AircraftTypeIcao { get; set; }

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
            Callsign = slot.Callsign;
            AircraftTypeIcao = slot.AircraftTypeIcao;
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
        return eventt.Airspaces
            .SelectMany(x => x.Slots)
            .OrderBy(x => x.EnterAt)
            .ThenBy(x => x.LeaveAt)
            .Select(x => new EventSlotDto(x))
            .ToArray();
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
        return new(slot);
    }

    [HttpGet("mine")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventSlotNotFound>]
    public async Task<EventSlotDto> GetMine(Ulid eid)
    {
        var user = await this.GetUser();

        var slot = await DbContext.EventSlot
            .Include(x => x.EventAirspace)
                .ThenInclude(x => x.Event)
            .Include(x => x.Booking)
            .FirstOrDefaultAsync(f => f.EventAirspace!.Id == eid && f.Booking!.UserId == user.Id)
            ?? throw new ApiError.EventSlotNotFoundForUser(eid, user.Id);
        return new EventSlotDto(slot);
    }

    public record CreateEventSlotDto
    {
        public required Ulid AirspaceId { get; set; }
        public required DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset? LeaveAt { get; set; }
        public string? Callsign { get; set; }
        public string? AircraftTypeIcao { get; set; }
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    [ApiError.Has<ApiError.EventAirspaceNotFound>]
    public async Task<EventSlotDto> Create(Ulid eid, CreateEventSlotDto dto)
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
        return new(slot);
    }

    public record UpdateEventSlotDto
    {
        public required DateTimeOffset EnterAt { get; set; }
        public DateTimeOffset? LeaveAt { get; set; }
        public string? Callsign { get; set; }
        public string? AircraftTypeIcao { get; set; }
    }

    [HttpPut("{sid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Update(Ulid eid, Ulid sid, UpdateEventSlotDto dto)
    {
        var slot = await LoadAsync(eid, sid);
        slot.EnterAt = dto.EnterAt;
        slot.LeaveAt = dto.LeaveAt;
        slot.Callsign = dto.Callsign;
        slot.AircraftTypeIcao = dto.AircraftTypeIcao;
        await DbContext.SaveChangesAsync();
        return new(slot);
    }

    [HttpDelete("{sid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventSlotDto> Delete(Ulid eid, Ulid sid)
    {
        var slot = await LoadAsync(eid, sid);
        DbContext.EventSlot.Remove(slot);
        await DbContext.SaveChangesAsync();
        return new(slot);
    }
}
