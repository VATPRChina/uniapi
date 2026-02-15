using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events/{eid}/airspaces")]
public class EventAirspaceController(DatabaseAdapter DbContext) : ControllerBase
{
    protected async Task<EventAirspace> LoadAsync(Ulid eid, Ulid aid)
    {
        var airspace = await DbContext.EventAirspace
            .Include(a => a.Event)
            .SingleOrDefaultAsync(a => a.Id == aid && a.EventId == eid)
            ?? throw new ApiError.EventAirspaceNotFound(eid, aid);
        return airspace;
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventAirspaceDto>> List(Ulid eid)
    {
        var eventt = await DbContext.Event.SingleOrDefaultAsync(x => x.Id == eid)
            ?? throw new ApiError.EventNotFound(eid);
        return await DbContext.Entry(eventt)
            .Collection(x => x.Airspaces!)
            .Query()
            .OrderBy(x => x.Name)
            .Select(x => EventAirspaceDto.From(x)).ToListAsync();
    }

    [HttpGet("{aid}")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventNotFound>]
    [ApiError.Has<ApiError.EventAirspaceNotFound>]
    public async Task<EventAirspaceDto> Get(Ulid eid, Ulid aid)
    {
        var airspace = await LoadAsync(eid, aid);
        return EventAirspaceDto.From(airspace);
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Create(Ulid eid, EventAirspaceSaveRequest dto)
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
        return EventAirspaceDto.From(airspace);
    }

    [HttpPut("{aid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Update(Ulid eid, Ulid aid, EventAirspaceSaveRequest dto)
    {
        var airspace = await LoadAsync(eid, aid);
        airspace.Name = dto.Name;
        airspace.IcaoCodes = dto.IcaoCodes.ToList();
        airspace.Description = dto.Description;
        await DbContext.SaveChangesAsync();
        return EventAirspaceDto.From(airspace);
    }

    [HttpDelete("{aid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventAirspaceDto> Delete(Ulid eid, Ulid aid)
    {
        var airspace = await LoadAsync(eid, aid);
        DbContext.EventAirspace.Remove(airspace);
        await DbContext.SaveChangesAsync();
        return EventAirspaceDto.From(airspace);
    }
}
