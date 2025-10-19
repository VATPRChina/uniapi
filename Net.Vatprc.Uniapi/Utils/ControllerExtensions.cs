using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;

namespace Net.Vatprc.Uniapi.Utils;

public static class ControllerExtensions
{
    public static Ulid GetUserId(this ControllerBase controller)
    {
        if (!controller.User.IsInRole("user"))
        {
            throw new ApiError.Forbidden(["user"]);
        }
        return Ulid.Parse(controller.User.Claims.FirstOrDefault(x => x.Type == ClaimTypes.NameIdentifier)?.Value);
    }
}
