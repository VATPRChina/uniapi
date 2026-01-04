using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Sector information.
/// </summary>
[ApiController, Route("api/sectors")]
public class SectorController(
    Database database,
    IUserAccessor userAccessor) : ControllerBase
{
    public record SectorPermissionResponse(
        bool HasPermission,
        string SectorType
    );

    [HttpGet("current/permission")]
    [Produces<SectorPermissionResponse>]
    public async Task<SectorPermissionResponse> GetPermission()
    {
        var user = await userAccessor.GetUser();

        var permissions = await database.UserAtcPermission.Where(u => u.UserId == user.Id).ToListAsync();

        var hasPermission = permissions.Any(p => p.CanOnline)
            || user.Cid == "1573922";

        // TODO: support visiting controller
        return new SectorPermissionResponse(hasPermission, "controller");
    }
}
