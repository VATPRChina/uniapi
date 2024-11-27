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
    public record UserDto(
        Ulid Id,
        string Cid,
        string FullName,
        DateTimeOffset CreatedAt,
        DateTimeOffset UpdatedAt,
        ISet<string> Roles
    )
    {
        public UserDto(User user) : this(user.Id, user.Cid, user.FullName, user.CreatedAt, user.UpdatedAt, user.Roles.ToHashSet())
        {
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
