using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;

namespace Net.Vatprc.Uniapi.Utils;

public static class ControllerExtensions
{
    public static Ulid GetUserId(this ControllerBase controller)
    {
        return Ulid.Parse(controller.User.Claims.FirstOrDefault(x => x.Type == ClaimTypes.NameIdentifier)?.Value);
    }
}
