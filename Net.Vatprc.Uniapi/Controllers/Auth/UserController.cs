using System.Security.Claims;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
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
        public UserDto(User user, bool showFullName = false) : this(
            user.Id,
            user.Cid,
            showFullName ? user.FullName : string.Empty,
            user.CreatedAt,
            user.UpdatedAt,
            null!,
            user.Roles.ToHashSet())
        {
            Roles = UserRoleService.GetRoleClosure(user.Roles);
        }
    }

    [HttpGet]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<IEnumerable<UserDto>> List()
    {
        var isStaff = await this.HasCurrentUserRole(UserRoles.Staff);
        return await DbContext.User.OrderBy(u => u.Cid).Select(x => new UserDto(x, isStaff)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<UserDto> Get(Ulid id)
    {
        return new UserDto(
            await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id),
            showFullName: await this.HasCurrentUserRole(UserRoles.Staff));
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
        return new UserDto(user);
    }

    [HttpPut("{id}/roles")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = UserRoles.Staff)]
    public async Task<UserDto> SetRoles(Ulid id, ISet<string> roles)
    {
        var currentUser = await this.GetUser();

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
        return new UserDto(user);
    }

    [HttpGet("{id}/atc/permissions")]
    [Authorize(Roles = UserRoles.ControllerTrainingMentor)]
    public async Task<IEnumerable<AtcPermissionDto>> GetAtcPermissions(Ulid id)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .Select(p => new AtcPermissionDto(p))
            .ToListAsync();
    }

    [HttpGet("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = UserRoles.ControllerTrainingMentor)]
    public async Task<AtcPermissionDto> GetAtcPermissionForKind(Ulid id, string kind)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .Select(p => new AtcPermissionDto(p))
            .FirstOrDefaultAsync() ?? throw new ApiError.UserAtcPermissionNotFound(id, kind);
    }

    public record SetAtcPermissionDto
    {
        public required UserAtcPermission.UserControllerState State { get; set; }
        public DateTimeOffset? SoloExpiresAt { get; set; }
    }

    [HttpPut("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
    public async Task<AtcPermissionDto> SetAtcPermissionForKind(Ulid id, string kind, SetAtcPermissionDto req)
    {
        if (req.State == UserAtcPermission.UserControllerState.Solo && req.SoloExpiresAt == null)
        {
            throw new ApiError.SoloExpirationNotProvided();
        }

        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var atcPermission = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .FirstOrDefaultAsync();
        if (atcPermission == null)
        {
            atcPermission = new UserAtcPermission
            {
                UserId = user.Id,
                PositionKindId = kind,
            };
            DbContext.UserAtcPermission.Add(atcPermission);
        }

        atcPermission.State = req.State;
        atcPermission.SoloExpiresAt = req.SoloExpiresAt;

        await DbContext.SaveChangesAsync();
        return new AtcPermissionDto(atcPermission);
    }

    [HttpDelete("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
    public async Task<IActionResult> DeleteAtcPermissionForKind(Ulid id, string kind)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var atcPermission = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .FirstOrDefaultAsync() ?? throw new ApiError.UserAtcPermissionNotFound(id, kind);

        DbContext.UserAtcPermission.Remove(atcPermission);
        await DbContext.SaveChangesAsync();
        return NoContent();
    }

    [HttpGet("me")]
    public async Task<UserDto> Me()
    {
        var subject = User.FindFirstValue(ClaimTypes.NameIdentifier);
        var userId = Ulid.Parse(subject);
        var user = await DbContext.User.FindAsync(userId);
        return new UserDto(user ?? throw new ApiError.UserNotFound(userId));
    }

    public record AtcPermissionDto(
        string PositionKindId,
        UserAtcPermission.UserControllerState State,
        DateTimeOffset? SoloExpiresAt
    )
    {
        public AtcPermissionDto(UserAtcPermission permission) : this(
            permission.PositionKindId,
            permission.State,
            permission.SoloExpiresAt)
        {
        }
    }

    [HttpGet("me/atc/permissions")]
    public async Task<IEnumerable<AtcPermissionDto>> GetAtcPermissions()
    {
        var user = await this.GetUser();

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .Select(p => new AtcPermissionDto(p))
            .ToListAsync();
    }
}
