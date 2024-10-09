using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Utils;
using System.Diagnostics.CodeAnalysis;
using Microsoft.AspNetCore.Authorization;

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
    ];

    protected IEnumerable<VatprcAtcService.Role> FlattenRoles(IEnumerable<VatprcAtcService.Role> Roles)
    {
        return Roles.SelectMany(r => FlattenRoles(r.AllSuperroles)).Concat(Roles);
    }

    [HttpGet("current/permission")]
    [AllowAnonymous]
    public async Task<SectorPermissionResponse> GetPermission()
    {
        var user = await DbContext.User.FindAsync(this.GetUserId()) ??
            throw new ApiError.UserNotFound(this.GetUserId());

        var roles = await VatprcAtcService.GetUserRole(user.Cid);
        var flattenRoles = FlattenRoles(roles);
        Logger.LogInformation("User {Cid} has roles {Roles}", user.Cid,
            string.Join(", ", flattenRoles.Select(r => r.Name)));
        var hasPermission = flattenRoles.Any(r => AllowedRoles.Contains(r.Name));

        return new SectorPermissionResponse(hasPermission);
    }
}
