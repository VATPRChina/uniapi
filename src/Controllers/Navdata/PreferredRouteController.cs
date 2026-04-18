using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto.Navdata;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Controllers.Navdata;

[ApiController]
[Route("api/navdata/preferred-routes")]
[Authorize(Roles = UserRoles.Volunteer)]
public class PreferredRouteController(
    DatabaseAdapter database
) : Controller
{
    [HttpGet]
    public async Task<IEnumerable<PreferredRouteDto>> GetPreferredRoutes()
    {
        throw new NotImplementedException();
    }

    [HttpGet("{id}")]
    public async Task<PreferredRouteDto> GetById(Ulid id)
    {
        throw new NotImplementedException();
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> CreatePreferredRoute(PreferredRouteSaveRequest dto)
    {
        throw new NotImplementedException();
    }

    [HttpPut("{id}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> UpdatePreferredRoute(Ulid id, PreferredRouteSaveRequest dto)
    {
        throw new NotImplementedException();
    }

    [HttpDelete("{id}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> DeletePreferredRoute(Ulid id)
    {
        throw new NotImplementedException();
    }
}
