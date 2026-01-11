using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Models.Atc;
using Net.Vatprc.Uniapi.Utils;
using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController]
[Route("api/users")]
public class UserAtcPermissionController(
    Database DbContext,
    IUserAccessor userAccessor) : Controller
{
    protected const string ADMIN_ROLES = $"{UserRoles.ControllerTrainingMentor},{UserRoles.ControllerTrainingDirectorAssistant}";
    protected static readonly IEnumerable<string> ALLOWED_RATINGS = ["OBS", "S1", "S2", "S3", "C1", "C3", "I1", "I3"];

    [HttpGet("me/atc/status")]
    public async Task<AtcStatusDto> GetAtcStatus()
    {
        var user = await userAccessor.GetUser();

        return await GetAtcStatus(user.Id);
    }

    [HttpGet("{id}/atc/status")]
    [Authorize(Roles = ADMIN_ROLES)]
    public async Task<AtcStatusDto> GetAtcStatus(Ulid id)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        var status = await DbContext.UserAtcStatus.SingleOrDefaultAsync(s => s.UserId == user.Id);

        var permissions = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .ToListAsync();

        return AtcStatusDto.From(user, status, permissions);
    }

    [HttpPut("{id}/atc/status")]
    [Authorize(Roles = ADMIN_ROLES)]
    public async Task<AtcStatusDto> SetAtcStatus(Ulid id, AtcStatusRequest req)
    {
        if (!ALLOWED_RATINGS.Contains(req.Rating))
        {
            throw new ApiError.InvalidAtcRating(req.Rating);
        }

        if (req.Permissions.Any(p => p.State == UserControllerState.Solo && p.SoloExpiresAt == null))
        {
            throw new ApiError.SoloExpirationNotProvided();
        }

        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        var status = await DbContext.UserAtcStatus.SingleOrDefaultAsync(s => s.UserId == user.Id);
        if (status == null)
        {
            status = new UserAtcStatus
            {
                UserId = user.Id,
                IsVisiting = req.IsVisiting,
                IsAbsent = req.IsAbsent,
                Rating = req.Rating,
            };
            DbContext.UserAtcStatus.Add(status);
        }
        else
        {
            status.IsVisiting = req.IsVisiting;
            status.IsAbsent = req.IsAbsent;
            status.Rating = req.Rating;
        }

        var permissions = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .ToListAsync();

        DbContext.UserAtcPermission.RemoveRange(permissions);
        permissions = req.Permissions.Select(p => new UserAtcPermission
        {
            UserId = user.Id,
            PositionKindId = p.PositionKindId,
            State = p.State,
            SoloExpiresAt = p.SoloExpiresAt,
        }).ToList();
        DbContext.UserAtcPermission.AddRange(permissions);

        await DbContext.SaveChangesAsync();

        return AtcStatusDto.From(user, status, permissions);
    }

    [HttpDelete("{id}/atc/status")]
    [Authorize(Roles = ADMIN_ROLES)]
    public async Task<IActionResult> DeleteAtcStatus(Ulid id)
    {
        var user = await DbContext.User.FindAsync(id) ?? throw new ApiError.UserNotFound(id);

        var status = await DbContext.UserAtcStatus.SingleOrDefaultAsync(s => s.UserId == user.Id)
            ?? throw new ApiError.NotFound(nameof(UserAtcStatus), id);

        DbContext.UserAtcStatus.Remove(status);

        var permissions = await DbContext.UserAtcPermission
            .Where(p => p.UserId == user.Id)
            .ToListAsync();

        DbContext.UserAtcPermission.RemoveRange(permissions);

        await DbContext.SaveChangesAsync();
        return NoContent();
    }
}
