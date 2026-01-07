using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController, Route("api/atc/controllers")]
public class AtcController(
    Database DbContext) : Controller
{
    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<ControllerDto>> List()
    {
        var permissions = await DbContext.UserAtcPermission
            .Include(p => p.User!)
                .ThenInclude(u => u.AtcStatus)
            .GroupBy(p => p.UserId)
            .ToListAsync();
        return permissions
            .Select(g => new ControllerDto
            {
                User = UserDto.From(g.First().User!, showFullName: true),
                Permissions = g.Select(p => AtcPermissionDto.From(p)),
                IsVisiting = g.First().User!.AtcStatus?.IsVisiting ?? false,
                IsAbsent = g.First().User!.AtcStatus?.IsAbsent ?? false,
            });
    }
}
