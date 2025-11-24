using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Dto;

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
