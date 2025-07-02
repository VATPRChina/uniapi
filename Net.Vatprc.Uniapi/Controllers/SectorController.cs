using System.Diagnostics.CodeAnalysis;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Sector information.
/// </summary>
[ApiController, Route("api/sectors")]
public class SectorController(
    VATPRCContext DbContext,
    ILogger<SectorController> Logger,
    VatprcAtcService VatprcAtcService) : ControllerBase
{
    public record SectorPermissionResponse
    {
        public required bool HasPermission { get; set; }
        public required string SectorType { get; set; }

        [SetsRequiredMembers]
        public SectorPermissionResponse(bool hasPermission, string sectorType)
        {
            HasPermission = hasPermission;
            SectorType = sectorType;
        }
    }

    static readonly IEnumerable<string> AllowedRoles =
    [
        "Online Permission",
        "ATC Student",
    ];

    protected IEnumerable<VatprcAtcService.Role> FlattenRoles(IEnumerable<VatprcAtcService.Role> Roles)
    {
        return Roles.SelectMany(r => FlattenRoles(r.AllSuperroles)).Concat(Roles);
    }

    [HttpGet("current/permission")]
    [Produces<SectorPermissionResponse>]
    public async Task<SectorPermissionResponse> GetPermission()
    {
        var user = await DbContext.User.FindAsync(this.GetUserId()) ??
            throw new ApiError.UserNotFound(this.GetUserId());

        var roles = await VatprcAtcService.GetUserRole(user.Cid);
        var flattenRoles = FlattenRoles(roles);
        Logger.LogInformation("User {Cid} has roles {Roles}", user.Cid,
            string.Join(", ", flattenRoles.Select(r => r.Name)));
        var hasPermission = flattenRoles.Any(r => AllowedRoles.Contains(r.Name))
            || user.Cid == "1676022"
            || user.Cid == "1573922";

        return new SectorPermissionResponse(hasPermission, flattenRoles.Any(r => r.Name == "Visiting Controller") ? "visiting_controller" : "controller");
    }
}
