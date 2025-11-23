using System.Security.Claims;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Auth;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/users")]
public class UserController(
    Database DbContext,
    IUserAccessor userAccessor) : ControllerBase
{
    [HttpGet]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<IEnumerable<UserDto>> List()
    {
        var isStaff = await userAccessor.HasCurrentUserRole(UserRoles.Staff);
        return await DbContext.User.OrderBy(u => u.Cid).Select(x => new UserDto(x, isStaff)).ToListAsync();
    }

    [HttpGet("{id}")]
    [ApiError.Has<ApiError.UserNotFound>]
    [Authorize(Roles = UserRoles.Volunteer)]
    public async Task<UserDto> Get(Ulid id)
    {
        return new UserDto(
            await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id),
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
        return new UserDto(user);
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

    public record UserDto(
    Ulid Id,
    string Cid,
    string FullName,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt,
    ISet<UserRoleDto> Roles,
    ISet<UserRoleDto> DirectRoles
)
    {
        public UserDto(User user, bool showFullName = false) : this(
            user.Id,
            user.Cid,
            showFullName ? user.FullName : string.Empty,
            user.CreatedAt,
            user.UpdatedAt,
            null!,
            user.Roles.Select(ConvertRole).ToHashSet())
        {
            Roles = UserRoleService.GetRoleClosure(user.Roles).Select(ConvertRole).ToHashSet();
        }

        public static UserRoleDto ConvertRole(string role) => role switch
        {
            UserRoles.Staff => UserRoleDto.Staff,
            UserRoles.Volunteer => UserRoleDto.Volunteer,
            UserRoles.DivisionDirector => UserRoleDto.DivisionDirector,
            UserRoles.ControllerTrainingDirector => UserRoleDto.ControllerTrainingDirector,
            UserRoles.ControllerTrainingDirectorAssistant => UserRoleDto.ControllerTrainingDirectorAssistant,
            UserRoles.ControllerTrainingInstructor => UserRoleDto.ControllerTrainingInstructor,
            UserRoles.ControllerTrainingMentor => UserRoleDto.ControllerTrainingMentor,
            UserRoles.ControllerTrainingSopEditor => UserRoleDto.ControllerTrainingSopEditor,
            UserRoles.OperationDirector => UserRoleDto.OperationDirector,
            UserRoles.OperationDirectorAssistant => UserRoleDto.OperationDirectorAssistant,
            UserRoles.OperationSectorEditor => UserRoleDto.OperationSectorEditor,
            UserRoles.OperationLoaEditor => UserRoleDto.OperationLoaEditor,
            UserRoles.EventDirector => UserRoleDto.EventDirector,
            UserRoles.EventCoordinator => UserRoleDto.EventCoordinator,
            UserRoles.EventGraphicsDesigner => UserRoleDto.EventGraphicsDesigner,
            UserRoles.TechDirector => UserRoleDto.TechDirector,
            UserRoles.TechDirectorAssistant => UserRoleDto.TechDirectorAssistant,
            UserRoles.TechAfvFacilityEngineer => UserRoleDto.TechAfvFacilityEngineer,
            UserRoles.Controller => UserRoleDto.Controller,
            UserRoles.ApiClient => UserRoleDto.ApiClient,
            UserRoles.User => UserRoleDto.User,
            _ => throw new ArgumentOutOfRangeException(nameof(role), $"Unknown role: {role}"),
        };
    }

    public enum UserRoleDto
    {
        Staff,
        Volunteer,
        DivisionDirector,
        ControllerTrainingDirector,
        ControllerTrainingDirectorAssistant,
        ControllerTrainingInstructor,
        ControllerTrainingMentor,
        ControllerTrainingSopEditor,
        OperationDirector,
        OperationDirectorAssistant,
        OperationSectorEditor,
        OperationLoaEditor,
        EventDirector,
        EventCoordinator,
        EventGraphicsDesigner,
        TechDirector,
        TechDirectorAssistant,
        TechAfvFacilityEngineer,
        Controller,
        ApiClient,
        User,
    }
}
