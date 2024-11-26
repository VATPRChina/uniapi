using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using System.Collections.Immutable;
using System.Security.Claims;
using System.Diagnostics.CodeAnalysis;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/users")]
public class UserController(VATPRCContext DbContext) : ControllerBase
{
    public record UserDto
    {
        public required Ulid Id { get; init; }
        public required string Cid { get; init; }
        public required string FullName { get; init; }
        public required DateTimeOffset CreatedAt { get; init; }
        public required DateTimeOffset UpdatedAt { get; init; }
        public required ISet<string> Roles { get; init; }

        [SetsRequiredMembers]
        public UserDto(User user)
        {
            Id = user.Id;
            Cid = user.Cid;
            FullName = user.FullName;
            CreatedAt = user.CreatedAt;
            UpdatedAt = user.UpdatedAt;
            Roles = user.Roles.ToHashSet();
            if (user.Roles.Contains(Models.User.UserRoles.Admin))
            {
                Roles.Add(Models.User.UserRoles.EventCoordinator);
                Roles.Add(Models.User.UserRoles.Controller);
            }
        }
    }

    [HttpGet]
    [Authorize(Roles = Models.User.UserRoles.Admin)]
    public async Task<IEnumerable<UserDto>> List()
    {
        return await DbContext.User.Select(x => new UserDto(x)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = Models.User.UserRoles.Admin)]
    public async Task<UserDto> Get(Ulid id)
    {
        return new UserDto(await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id));
    }

    [HttpGet("me")]
    public async Task<UserDto> Me()
    {
        var subject = User.FindFirstValue(ClaimTypes.NameIdentifier);
        var userId = Ulid.Parse(subject);
        var user = await DbContext.User.FindAsync(userId);
        return new UserDto(user ?? throw new ApiError.UserNotFound(userId));
    }
}
