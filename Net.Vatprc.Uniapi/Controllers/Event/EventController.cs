using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
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
        public DateTimeOffset StartBookingAt { get; init; }
        public DateTimeOffset EndBookingAt { get; init; }
        public string? ImageUrl { get; set; }
        public string Description { get; set; }

        public EventDto(Event eventt)
        {
            Id = eventt.Id;
            CreatedAt = eventt.CreatedAt;
            UpdatedAt = eventt.UpdatedAt;
            Title = eventt.Title;
            StartAt = eventt.StartAt;
            EndAt = eventt.EndAt;
            StartBookingAt = eventt.StartBookingAt;
            EndBookingAt = eventt.EndBookingAt;
            ImageUrl = eventt.ImageUrl;
            Description = eventt.Description;
        }
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<EventDto>> List()
    {
        var query = DbContext.Event.AsQueryable()
            .Where(x => DateTimeOffset.UtcNow < x.EndAt);
        if (!User.IsInRole(Models.User.UserRoles.EventCoordinator))
        {
            query = query.Where(x => DateTimeOffset.UtcNow.AddDays(7) > x.StartBookingAt);
        }

        return await query
            .OrderBy(x => x.StartBookingAt)
            .Select(x => new EventDto(x)).ToListAsync();
    }

    [HttpGet("{eid}")]
    [AllowAnonymous]
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
        public required DateTimeOffset StartBookingAt { get; init; }
        public required DateTimeOffset EndBookingAt { get; init; }
        public string? ImageUrl { get; init; }
        public required string Description { get; set; }
    }

    [HttpPost]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventDto> Create(CreateEventDto dto)
    {
        var eventt = new Event()
        {
            Title = dto.Title,
            StartAt = dto.StartAt.ToUniversalTime(),
            EndAt = dto.EndAt.ToUniversalTime(),
            StartBookingAt = dto.StartBookingAt.ToUniversalTime(),
            EndBookingAt = dto.EndBookingAt.ToUniversalTime(),
            ImageUrl = dto.ImageUrl,
            Description = dto.Description,
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
        public required DateTimeOffset StartBookingAt { get; init; }
        public required DateTimeOffset EndBookingAt { get; init; }
        public string? ImageUrl { get; init; }
        public required string Description { get; init; }
    }

    [HttpPost("{eid}")]
    [Authorize(Roles = Models.User.UserRoles.EventCoordinator)]
    public async Task<EventDto> Update(Ulid eid, UpdateEventDto dto)
    {
        var eventt = await DbContext.Event.FindAsync(eid) ?? throw new ApiError.EventNotFound(eid);
        eventt.Title = dto.Title;
        eventt.StartAt = dto.StartAt.ToUniversalTime();
        eventt.EndAt = dto.EndAt.ToUniversalTime();
        eventt.StartBookingAt = dto.StartBookingAt.ToUniversalTime();
        eventt.EndBookingAt = dto.EndBookingAt.ToUniversalTime();
        eventt.ImageUrl = dto.ImageUrl;
        eventt.Description = dto.Description;
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
