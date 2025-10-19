namespace Net.Vatprc.Uniapi.Services;

public class UserRoleService
{
    public static IEnumerable<string> GetDirectlyInheritedRoles(string role)
    {
        var roles = new HashSet<string> { role };
        if (role == Models.User.UserRoles.Admin)
        {
            return [
                Models.User.UserRoles.EventCoordinator,
                Models.User.UserRoles.Controller,
                Models.User.UserRoles.Editor,
            ];
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
