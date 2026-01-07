using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/users")]
public class UserAtcPermissionController(
    Database DbContext,
    IUserAccessor userAccessor) : Controller
{

    [HttpGet("me/atc/permissions")]
    public async Task<IEnumerable<AtcPermissionDto>> GetAtcPermissions()
    {
        var user = await userAccessor.GetUser();

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .Select(p => AtcPermissionDto.From(p))
            .ToListAsync();
    }

    [HttpGet("{id}/atc/status")]
    public async Task<ControllerDto> GetAtcStatus(Ulid id)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        var status = await DbContext.UserAtcStatus.SingleOrDefaultAsync(s => s.UserId == user.Id);

        var atcPermissions = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .ToListAsync();

        return new ControllerDto
        {
            User = UserDto.From(user),
            Permissions = atcPermissions.Select(AtcPermissionDto.From),
            IsVisiting = status?.IsVisiting ?? false,
            IsAbsent = status?.IsAbsent ?? false,
        };
    }

    [HttpGet("{id}/atc/permissions")]
    [Authorize(Roles = $"{UserRoles.ControllerTrainingMentor},{UserRoles.ControllerTrainingDirectorAssistant}")]
    public async Task<IEnumerable<AtcPermissionDto>> GetAtcPermissions(Ulid id)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .Select(p => AtcPermissionDto.From(p))
            .ToListAsync();
    }

    [HttpGet("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = $"{UserRoles.ControllerTrainingMentor},{UserRoles.ControllerTrainingDirectorAssistant}")]
    public async Task<AtcPermissionDto> GetAtcPermissionForKind(Ulid id, string kind)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        return await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .Select(p => AtcPermissionDto.From(p))
            .FirstOrDefaultAsync() ?? throw new ApiError.UserAtcPermissionNotFound(id, kind);
    }

    [HttpPut("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
    public async Task<AtcPermissionDto> SetAtcPermissionForKind(Ulid id, string kind, AtcPermissionSetRequest req)
    {
        if (req.State == UserAtcPermission.UserControllerState.Solo && req.SoloExpiresAt == null)
        {
            throw new ApiError.SoloExpirationNotProvided();
        }

        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var atcPermission = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .FirstOrDefaultAsync();
        if (atcPermission == null)
        {
            atcPermission = new UserAtcPermission
            {
                UserId = user.Id,
                PositionKindId = kind,
            };
            DbContext.UserAtcPermission.Add(atcPermission);
        }

        atcPermission.State = req.State;
        atcPermission.SoloExpiresAt = req.SoloExpiresAt;

        await DbContext.SaveChangesAsync();
        return AtcPermissionDto.From(atcPermission);
    }

    [HttpDelete("{id}/atc/permissions/{kind}")]
    [Authorize(Roles = UserRoles.ControllerTrainingDirectorAssistant)]
    public async Task<IActionResult> DeleteAtcPermissionForKind(Ulid id, string kind)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);
        var atcPermission = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id && p.PositionKindId == kind)
            .FirstOrDefaultAsync() ?? throw new ApiError.UserAtcPermissionNotFound(id, kind);

        DbContext.UserAtcPermission.Remove(atcPermission);
        await DbContext.SaveChangesAsync();
        return NoContent();
    }
}
