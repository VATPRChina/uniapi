using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Utils;

public static class ControllerExtensions
{
    public static Ulid GetUserId(this ControllerBase controller)
    {
        if (!controller.User.IsInRole(UserRoles.User))
        {
            throw new ApiError.Forbidden([UserRoles.User]);
        }
        return Ulid.Parse(controller.User.Claims.FirstOrDefault(x => x.Type == ClaimTypes.NameIdentifier)?.Value);
    }

    public static async Task<User> GetUser(this ControllerBase controller)
    {
        var dbContext = controller.HttpContext.RequestServices.GetRequiredService<VATPRCContext>();
        return await dbContext.User.FindAsync(controller.GetUserId()) ??
            throw new ApiError.UserNotFound(controller.GetUserId());
    }

    public static async Task<bool> HasCurrentUserRole(this ControllerBase controller, string role)
    {
        var dbContext = controller.HttpContext.RequestServices.GetRequiredService<VATPRCContext>();
        var user = await dbContext.User.FindAsync(controller.GetUserId()) ??
            throw new ApiError.UserNotFound(controller.GetUserId());
        var roles = UserRoleService.GetRoleClosure(user.Roles);
        return roles.Contains(role);
    }
}
