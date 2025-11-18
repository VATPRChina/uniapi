using System.Security.Claims;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Utils;

public class UserAccessor(IHttpContextAccessor httpContextAccessor, Database dbContext)
{
    public Ulid GetUserId()
    {
        var httpContext = httpContextAccessor.HttpContext
            ?? throw new InvalidOperationException("No HttpContext available");
        if (!httpContext.User.IsInRole(UserRoles.User))
        {
            throw new ApiError.Forbidden([UserRoles.User]);
        }
        return Ulid.Parse(httpContext.User.Claims.FirstOrDefault(x => x.Type == ClaimTypes.NameIdentifier)?.Value);
    }

    public async Task<User> GetUser()
    {
        return await dbContext.User.FindAsync(GetUserId()) ??
            throw new ApiError.UserNotFound(GetUserId());
    }

    public async Task<bool> HasCurrentUserRole(string role)
    {
        var user = await dbContext.User.FindAsync(GetUserId()) ??
            throw new ApiError.UserNotFound(GetUserId());
        var roles = UserRoleService.GetRoleClosure(user.Roles);
        return roles.Contains(role);
    }
}
