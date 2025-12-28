using System.Collections.Immutable;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

public class UserRoleService
{
    protected static readonly ImmutableDictionary<string, IEnumerable<string>> DIRECTLY_INHERITED_ROLES =
        new Dictionary<string, IEnumerable<string>> {
            { UserRoles.Staff, [UserRoles.Volunteer] },
            { UserRoles.Volunteer, [] },

            { UserRoles.DivisionDirector, [
                UserRoles.Staff,
                UserRoles.ControllerTrainingDirector,
                UserRoles.OperationDirector,
                UserRoles.EventDirector,
                UserRoles.TechDirector
            ] },

            { UserRoles.ControllerTrainingDirector, [
                UserRoles.Staff,
                UserRoles.ControllerTrainingDirectorAssistant,
                UserRoles.ControllerTrainingInstructor,
                UserRoles.ControllerTrainingMentor,
                UserRoles.ControllerTrainingSopEditor,
            ] },
            { UserRoles.ControllerTrainingDirectorAssistant, [UserRoles.Volunteer] },
            { UserRoles.ControllerTrainingInstructor, [UserRoles.Volunteer, UserRoles.ControllerTrainingMentor] },
            { UserRoles.ControllerTrainingMentor, [UserRoles.Volunteer] },
            { UserRoles.ControllerTrainingSopEditor, [UserRoles.Volunteer] },

            { UserRoles.CommunityDirector, [UserRoles.Staff]},

            { UserRoles.OperationDirector, [
                UserRoles.Staff,
                UserRoles.OperationDirectorAssistant,
                UserRoles.OperationSectorEditor,
                UserRoles.OperationLoaEditor,
            ] },
            { UserRoles.OperationDirectorAssistant, [UserRoles.Volunteer] },
            { UserRoles.OperationSectorEditor, [UserRoles.Volunteer] },
            { UserRoles.OperationLoaEditor, [UserRoles.Volunteer] },

            { UserRoles.EventDirector, [
                UserRoles.Staff,
                UserRoles.EventCoordinator,
                UserRoles.EventGraphicsDesigner,
            ] },
            { UserRoles.EventCoordinator, [UserRoles.Volunteer] },
            { UserRoles.EventGraphicsDesigner, [UserRoles.Volunteer] },

            { UserRoles.TechDirector, [
                UserRoles.Staff,
                UserRoles.TechDirectorAssistant,
                UserRoles.TechAfvFacilityEngineer,
            ] },
            { UserRoles.TechDirectorAssistant, [UserRoles.Volunteer] },
            { UserRoles.TechAfvFacilityEngineer, [UserRoles.Volunteer] },

            { UserRoles.Controller, [] },
        }.ToImmutableDictionary();

    public static IEnumerable<string> GetDirectlyInheritedRoles(string role)
    {
        if (DIRECTLY_INHERITED_ROLES.TryGetValue(role, out var inheritedRoles))
        {
            return inheritedRoles;
        }
        return [];
    }

    private static void AddInheritedRolesRecursively(string role, ISet<string> closure)
    {
        var inheritedRoles = GetDirectlyInheritedRoles(role);
        foreach (var inheritedRole in inheritedRoles)
        {
            if (closure.Add(inheritedRole))
            {
                AddInheritedRolesRecursively(inheritedRole, closure);
            }
        }
    }

    public static ISet<string> GetRoleClosure(IEnumerable<string> roles)
    {
        var closure = new HashSet<string>(roles);
        foreach (var role in roles.ToList())
        {
            AddInheritedRolesRecursively(role, closure);
        }
        return closure;
    }
}
