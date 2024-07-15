using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;
using Net.Vatprc.Uniapi.Models;
using System.Collections;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/notams")]
public class NotamController(VATPRCContext DbContext) : ControllerBase
{
    public record NotamDto
    {
        public Ulid Id { get; set; }
        public string Title { get; set; } = string.Empty;
        public string Description { get; set; } = string.Empty;
        public DateTimeOffset EffectiveFrom { get; set; }
        public DateTimeOffset ExpireAfter { get; set; }
        public DateTimeOffset CreatedAt { get; set; }
        public DateTimeOffset UpdatedAt { get; set; }

        public NotamDto(Notam notam)
        {
            Id = notam.Id;
            Title = notam.Title;
            Description = notam.Description;
            EffectiveFrom = notam.EffectiveFrom;
            ExpireAfter = notam.ExpireAfter;
            CreatedAt = notam.CreatedAt;
            UpdatedAt = notam.UpdatedAt;
        }
    }

    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<NotamDto>> List()
    {
        return await DbContext.Notam
            .Where(x => x.EffectiveFrom <= DateTimeOffset.UtcNow && DateTimeOffset.UtcNow <= x.ExpireAfter)
            .Select(x => new NotamDto(x)).ToListAsync();
    }

    [HttpGet("{id}")]
    [AllowAnonymous]
    public async Task<ActionResult<NotamDto>> Get(Ulid id)
    {
        var notam = await DbContext.Notam.FindAsync(id)
            ?? throw new ApiError.NotamNotFound(id);
        return new NotamDto(notam);
    }
}
