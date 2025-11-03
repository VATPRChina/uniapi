using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models;

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
}
