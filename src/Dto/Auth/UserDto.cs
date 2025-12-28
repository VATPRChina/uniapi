using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Dto;

public record UserDto
{
    public required Ulid Id { get; init; }
    public required string Cid { get; init; }
    public required string FullName { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }
    public required ISet<UserRoleDto> Roles { get; init; }
    public required ISet<UserRoleDto> DirectRoles { get; init; }

    public static UserDto From(User user, bool showFullName = false, IEnumerable<string>? roles = null)
    {
        return new()
        {
            Id = user.Id,
            Cid = user.Cid,
            FullName = showFullName ? user.FullName : string.Empty,
            CreatedAt = user.CreatedAt,
            UpdatedAt = user.UpdatedAt,
            DirectRoles = user.Roles.Select(ConvertRole).ToHashSet(),
            Roles = (roles ?? UserRoleService.GetRoleClosure(user.Roles)).Select(ConvertRole).ToHashSet(),
        };
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
        UserRoles.CommunityDirector => UserRoleDto.CommunityDirector,
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
