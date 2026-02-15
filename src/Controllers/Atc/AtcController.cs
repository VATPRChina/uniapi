using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers.Atc;

[ApiController, Route("api/atc/controllers")]
public class AtcController(
    DatabaseAdapter DbContext) : Controller
{
    [HttpGet]
    [AllowAnonymous]
    public async Task<IEnumerable<AtcStatusDto>> List()
    {
        var permissions = await DbContext.UserAtcPermission
            .Include(p => p.User!)
                .ThenInclude(u => u.AtcStatus)
            .GroupBy(p => p.UserId)
            .ToListAsync();
        return permissions
            .Select(g => new AtcStatusDto
            {
                UserId = g.First().UserId,
                User = UserDto.From(g.First().User!, showFullName: true),
                Permissions = g.Select(p => AtcPermissionDto.From(p)),
                IsVisiting = g.First().User!.AtcStatus?.IsVisiting ?? false,
                IsAbsent = g.First().User!.AtcStatus?.IsAbsent ?? false,
                Rating = g.First().User!.AtcStatus?.Rating ?? "OBS",
            });
    }
}
