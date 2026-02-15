using System.Security.Claims;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Auth;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/users")]
public class UserController(
    DatabaseAdapter DbContext,
    IUserAccessor userAccessor) : ControllerBase
{
    [HttpGet]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<IEnumerable<UserDto>> List()
    {
        var isStaff = await userAccessor.HasCurrentUserRole(UserRoles.Staff);
        return await DbContext.User.OrderBy(u => u.Cid).Select(x => UserDto.From(x, isStaff)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<UserDto> Get(Ulid id)
    {
        return UserDto.From(
            await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id),
            showFullName: await userAccessor.HasCurrentUserRole(UserRoles.Staff));
    }

    [HttpGet("by-cid/{cid}")]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<UserDto?> GetByCid(string cid)
    {
        var user = await DbContext.User.FirstOrDefaultAsync(u => u.Cid == cid);
        if (user == null)
        {
            throw new ApiError.UserNotFoundByCid(cid);
        }

        return UserDto.From(
            user,
            showFullName: await userAccessor.HasCurrentUserRole(UserRoles.Staff));
    }

    [HttpPost("by-cid/{cid}")]
    [Authorize(Roles = UserRoles.Staff)]
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
        return UserDto.From(user);
    }

    [HttpPut("{id}/roles")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = UserRoles.Staff)]
    public async Task<UserDto> SetRoles(Ulid id, ISet<string> roles)
    {
        var currentUser = await userAccessor.GetUser();

        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var curRoles = UserRoleService.GetRoleClosure(user.Roles);
        var newRoles = UserRoleService.GetRoleClosure(roles);
        if (curRoles.Contains(UserRoles.Staff)
            && !newRoles.Contains(UserRoles.Staff)
            && !currentUser.Roles.Contains(UserRoles.DivisionDirector))
        {
            throw new ApiError.RemoveStaffForbidden();
        }
        user.Roles = roles.ToList();
        await DbContext.SaveChangesAsync();
        return UserDto.From(user);
    }

    [HttpGet("me")]
    public async Task<UserDto> Me()
    {
        var subject = User.FindFirstValue(ClaimTypes.NameIdentifier);
        var userId = Ulid.Parse(subject);
        var user = await DbContext.User.FindAsync(userId);
        return UserDto.From(user ?? throw new ApiError.UserNotFound(userId), true);
    }
}
