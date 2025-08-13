using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/airspaces")]
public class EventAirspaceController(VATPRCContext DbContext) : ControllerBase
{
    protected async Task<EventAirspace> LoadAsync(Ulid eid, Ulid aid)
    {
        var airspace = await DbContext.EventAirspace
            .Include(a => a.Event)
            .SingleOrDefaultAsync(a => a.Id == aid && a.EventId == eid)
            ?? throw new ApiError.EventAirspaceNotFound(eid, aid);
        return airspace;
    }

    public record EventAirspaceDto
    {
        public Ulid Id { get; init; }
        public Ulid EventId { get; init; }
        public string Name { get; init; }
        public DateTimeOffset CreatedAt { get; init; }
        public DateTimeOffset UpdatedAt { get; init; }
        public IEnumerable<string> IcaoCodes { get; init; }
        public string Description { get; set; }

        public EventAirspaceDto(EventAirspace airspace)
        {
            Id = airspace.Id;
            EventId = airspace.EventId;
            Name = airspace.Name;
            CreatedAt = airspace.CreatedAt;
            UpdatedAt = airspace.UpdatedAt;
            IcaoCodes = airspace.IcaoCodes;
            Description = airspace.Description;
        }
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventAirspaceDto>> List(Ulid eid)
    {
        var eventt = await DbContext.Event.SingleOrDefaultAsync(x => x.Id == eid)
            ?? throw new ApiError.EventNotFound(eid);
        return await DbContext.Entry(eventt)
            .Collection(x => x.Airspaces)
            .Query()
            .OrderBy(x => x.Name)
            .Select(x => new EventAirspaceDto(x)).ToListAsync();
    }

    [HttpGet("{aid}")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventNotFound>]
    [ApiError.Has<ApiError.EventAirspaceNotFound>]
    public async Task<EventAirspaceDto> Get(Ulid eid, Ulid aid)
    {
        var airspace = await LoadAsync(eid, aid);
        return new(airspace);
    }

    public record CreateEventAirspaceDto
    {
        public required string Name { get; set; }
        public required IEnumerable<string> IcaoCodes { get; set; }
        public required string Description { get; set; }
    }

    [HttpPost]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Create(Ulid eid, CreateEventAirspaceDto dto)
    {
        var airspace = new EventAirspace()
        {
            EventId = eid,
            Name = dto.Name,
            IcaoCodes = dto.IcaoCodes.ToList(),
            Description = dto.Description,
        };
        DbContext.EventAirspace.Add(airspace);
        await DbContext.SaveChangesAsync();
        return new(airspace);
    }

    public record UpdateEventAirspaceDto
    {
        public required string Name { get; set; }
        public required IEnumerable<string> IcaoCodes { get; set; }
        public required string Description { get; set; }
    }

    [HttpPut("{aid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Update(Ulid eid, Ulid aid, UpdateEventAirspaceDto dto)
    {
        var airspace = await LoadAsync(eid, aid);
        airspace.Name = dto.Name;
        airspace.IcaoCodes = dto.IcaoCodes.ToList();
        airspace.Description = dto.Description;
        await DbContext.SaveChangesAsync();
        return new(airspace);
    }

    [HttpDelete("{aid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Delete(Ulid eid, Ulid aid)
    {
        var airspace = await LoadAsync(eid, aid);
        DbContext.EventAirspace.Remove(airspace);
        await DbContext.SaveChangesAsync();
        return new(airspace);
    }
}
