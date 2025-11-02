using System.Security.Claims;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

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
        ISet<string> Roles,
        ISet<string> DirectRoles
    )
    {
        public UserDto(User user) : this(user.Id, user.Cid, user.FullName, user.CreatedAt, user.UpdatedAt, null!, user.Roles.ToHashSet())
        {
            Roles = UserRoleService.GetRoleClosure(user.Roles);
        }
    }

    [HttpGet]
    [Authorize(Roles = Models.User.UserRoles.Staff)]
    public async Task<IEnumerable<UserDto>> List()
    {
        return await DbContext.User.Select(x => new UserDto(x)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = Models.User.UserRoles.Staff)]
    public async Task<UserDto> Get(Ulid id)
    {
        return new UserDto(await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id));
    }

    [HttpPost("by-cid/{cid}")]
    [Authorize(Roles = Models.User.UserRoles.Staff)]
    public async Task<UserDto> AssumeByCid(string cid)
    {
        var user = new User
        {
            Cid = cid,
            FullName = cid,
            Email = null,
        };
        DbContext.User.Add(user);
        await DbContext.SaveChangesAsync();
        return new UserDto(user);
    }

    [HttpPut("{id}/roles")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = Models.User.UserRoles.Staff)]
    public async Task<UserDto> SetRoles(Ulid id, ISet<string> roles)
    {
        var currentUser = await this.GetUser();

        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var curRoles = UserRoleService.GetRoleClosure(user.Roles);
        var newRoles = UserRoleService.GetRoleClosure(roles);
        if (curRoles.Contains(Models.User.UserRoles.Staff)
            && !newRoles.Contains(Models.User.UserRoles.Staff)
            && !currentUser.Roles.Contains(Models.User.UserRoles.DivisionDirector))
        {
            throw new ApiError.RemoveStaffForbidden();
        }
        user.Roles = roles.ToList();
        await DbContext.SaveChangesAsync();
        return new UserDto(user);
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
