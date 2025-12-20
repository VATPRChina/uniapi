using System.Security.Claims;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Utils;

public interface IUserAccessor
{
    public Task<User> GetUser();
    public Ulid GetUserId();
    public Task<bool> HasCurrentUserRole(string role);
    public Task<bool> HasCurrentUserAnyRoleOf(params string[] expectedRoles);
}

public class UserAccessor(IHttpContextAccessor httpContextAccessor, Database dbContext) : IUserAccessor
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

    public async Task<bool> HasCurrentUserAnyRoleOf(params string[] expectedRoles)
    {
        var user = await dbContext.User.FindAsync(GetUserId()) ??
            throw new ApiError.UserNotFound(GetUserId());
        var roles = UserRoleService.GetRoleClosure(user.Roles);
        return roles.Any(r => expectedRoles.Contains(r));
    }
}
