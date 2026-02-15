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
[ApiController, Route("api/events")]
public class EventController(DatabaseAdapter DbContext) : ControllerBase
{
    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventDto>> List()
    {
        var query = DbContext.Event.AsQueryable()
            .Where(x => DateTimeOffset.UtcNow < x.EndAt);
        return await query
            .OrderBy(x => x.StartAt)
            .Select(x => EventDto.From(x)).ToListAsync();
    }

    [HttpGet("past")]
    [AllowAnonymous]
    public async Task<IEnumerable<EventDto>> ListPast(DateTimeOffset? until = null)
    {
        var query = DbContext.Event.AsQueryable()
            .Where(x => x.StartAt < DateTimeOffset.UtcNow
                && (until == null || x.StartAt <= until))
            .OrderByDescending(x => x.StartAt)
            .Select(x => EventDto.From(x));
        return await query.ToListAsync();
    }

    [HttpGet("{eid}")]
    [AllowAnonymous]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventDto> Get(Ulid eid)
    {
        return EventDto.From(await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid));
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventDto> Create(EventSaveRequest dto)
    {
        if (dto.StartBookingAt == null ^ dto.EndBookingAt == null)
        {
            throw new ApiError.BadRequest("Both StartBookingAt and EndBookingAt must be set or both must be null.");
        }

        var eventt = new Event()
        {
            Title = dto.Title,
            TitleEn = dto.TitleEn,
            StartAt = dto.StartAt.ToUniversalTime(),
            EndAt = dto.EndAt.ToUniversalTime(),
            StartBookingAt = dto.StartBookingAt?.ToUniversalTime(),
            EndBookingAt = dto.EndBookingAt?.ToUniversalTime(),
            StartAtcBookingAt = dto.StartAtcBookingAt?.ToUniversalTime(),
            ImageUrl = dto.ImageUrl,
            CommunityLink = dto.CommunityLink,
            VatsimLink = dto.VatsimLink,
            Description = dto.Description,
        };
        DbContext.Event.Add(eventt);
        await DbContext.SaveChangesAsync();
        return EventDto.From(eventt);
    }

    [HttpPost("{eid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventDto> Update(Ulid eid, EventSaveRequest dto)
    {
        if (dto.StartBookingAt == null ^ dto.EndBookingAt == null)
        {
            throw new ApiError.BadRequest("Both StartBookingAt and EndBookingAt must be set or both must be null.");
        }

        var eventt = await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid);
        eventt.Title = dto.Title;
        eventt.TitleEn = dto.TitleEn;
        eventt.StartAt = dto.StartAt.ToUniversalTime();
        eventt.EndAt = dto.EndAt.ToUniversalTime();
        eventt.StartBookingAt = dto.StartBookingAt?.ToUniversalTime();
        eventt.EndBookingAt = dto.EndBookingAt?.ToUniversalTime();
        eventt.StartAtcBookingAt = dto.StartAtcBookingAt?.ToUniversalTime();
        eventt.ImageUrl = dto.ImageUrl;
        eventt.CommunityLink = dto.CommunityLink;
        eventt.VatsimLink = dto.VatsimLink;
        eventt.Description = dto.Description;
        await DbContext.SaveChangesAsync();
        return EventDto.From(eventt);
    }

    [HttpDelete("{eid}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<EventDto> Delete(Ulid eid)
    {
        var eventt = await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid);
        DbContext.Event.Remove(eventt);
        await DbContext.SaveChangesAsync();
        return EventDto.From(eventt);
    }
}
