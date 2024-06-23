using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/events")]
public class EventController(VATPRCContext DbContext) : ControllerBase
{
    public record EventDto
    {
        public Ulid Id { get; init; }
        public DateTimeOffset CreatedAt { get; init; }
        public DateTimeOffset UpdatedAt { get; init; }
        public string Title { get; init; }
        public DateTimeOffset StartAt { get; init; }
        public DateTimeOffset EndAt { get; init; }

        public EventDto(Event eventt)
        {
            Id = eventt.Id;
            CreatedAt = eventt.CreatedAt;
            UpdatedAt = eventt.UpdatedAt;
            Title = eventt.Title;
            StartAt = eventt.StartAt;
            EndAt = eventt.EndAt;
        }
    }

    [HttpGet]
    public async Task<IEnumerable<EventDto>> List()
    {
        return await DbContext.Event.Select(x => new EventDto(x)).ToListAsync();
    }

    [HttpGet("{eid}")]
    [ApiError.Has<ApiError.EventNotFound>]
    public async Task<EventDto> Get(Ulid eid)
    {
        return new EventDto(await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid));
    }

    public record CreateEventDto
    {
        public required string Title { get; set; }
        public required DateTimeOffset StartAt { get; set; }
        public required DateTimeOffset EndAt { get; set; }
    }

    [HttpPost]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventDto> Create(CreateEventDto dto)
    {
        var eventt = new Event()
        {
            Title = dto.Title,
            StartAt = dto.StartAt,
            EndAt = dto.EndAt,
        };
        DbContext.Event.Add(eventt);
        await DbContext.SaveChangesAsync();
        return new(eventt);
    }

    public record UpdateEventDto
    {
        public required string Title { get; set; }
        public required DateTimeOffset StartAt { get; set; }
        public required DateTimeOffset EndAt { get; set; }
    }

    [HttpPost("{eid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventDto> Update(Ulid eid, UpdateEventDto dto)
    {
        var eventt = await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid);
        eventt.Title = dto.Title;
        eventt.StartAt = dto.StartAt;
        eventt.EndAt = dto.EndAt;
        await DbContext.SaveChangesAsync();
        return new(eventt);
    }

    [HttpDelete("{eid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventDto> Delete(Ulid eid)
    {
        var eventt = await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid);
        DbContext.Event.Remove(eventt);
        await DbContext.SaveChangesAsync();
        return new(eventt);
    }
}
