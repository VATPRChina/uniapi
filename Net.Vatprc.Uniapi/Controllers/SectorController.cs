using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Utils;
using System.Diagnostics.CodeAnalysis;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Sector information.
/// </summary>
[ApiController, Route("api/sectors")]
public class SectorController(
    VATPRCContext DbContext,
    VatsimService VatsimService) : ControllerBase
{
    public record SectorPermissionResponse
    {
        public required bool HasPermission { get; set; }

        [SetsRequiredMembers]
        public SectorPermissionResponse(bool hasPermission)
        {
            HasPermission = hasPermission;
        }
    }

    static readonly IEnumerable<string> AllowedRoles =
    [
        "Online Permission",
        "ATC Student",
        "S3 Controller",
        "S2 Controller",
        "S1 Controller",
        "C1 Controller",
        "C3 Controller",
    ];

    [HttpGet("current/permission")]
    public async Task<SectorPermissionResponse> GetPermission()
    {
        var user = await DbContext.User.FindAsync(this.GetUserId()) ??
            throw new ApiError.UserNotFound(this.GetUserId());

        // FIXME: This is a temporary solution to allow the user to access the sector
        if (user.Cid == "1638882")
        {
            return new SectorPermissionResponse(true);
        }

        var controllers = await VatsimService.GetAtcList();
        var atc = controllers.FirstOrDefault(c => c.Id.ToString() == user.Cid);
        if (atc == null)
        {
            return new SectorPermissionResponse(false);
        }
        var hasPermission = atc.Roles.Any(r => AllowedRoles.Contains(r.Name));
        return new SectorPermissionResponse(hasPermission);
    }
}
