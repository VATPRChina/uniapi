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
        return await database.PreferredRoute
            .Select(r => PreferredRouteDto.FromModel(r))
            .ToListAsync();
    }

    [HttpGet("{id}")]
    public async Task<PreferredRouteDto> GetById(Ulid id)
    {
        return await database.PreferredRoute
            .Where(r => r.Id == id)
            .Select(r => PreferredRouteDto.FromModel(r))
            .SingleAsync();
    }

    [HttpPost]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> CreatePreferredRoute(PreferredRouteSaveRequest dto)
    {
        var route = new Models.Navdata.PreferredRoute
        {
            Departure = dto.Departure,
            Arrival = dto.Arrival,
            RawRoute = dto.RawRoute,
            CruisingLevelRestriction = dto.CruisingLevelRestriction,
            AllowedAltitudes = dto.AllowedAltitudes,
            MinimalAltitude = dto.MinimalAltitude,
            Remarks = dto.Remarks,
            ValidFrom = dto.ValidFrom,
            ValidUntil = dto.ValidUntil,
        };

        database.PreferredRoute.Add(route);
        await database.SaveChangesAsync();

        return PreferredRouteDto.FromModel(route);
    }

    [HttpPut("{id}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> UpdatePreferredRoute(Ulid id, PreferredRouteSaveRequest dto)
    {
        var route = await database.PreferredRoute.FindAsync(id)
            ?? throw new ArgumentException("preferred route not found", nameof(id));

        route.Departure = dto.Departure;
        route.Arrival = dto.Arrival;
        route.RawRoute = dto.RawRoute;
        route.CruisingLevelRestriction = dto.CruisingLevelRestriction;
        route.AllowedAltitudes = dto.AllowedAltitudes;
        route.MinimalAltitude = dto.MinimalAltitude;
        route.Remarks = dto.Remarks;
        route.ValidFrom = dto.ValidFrom;
        route.ValidUntil = dto.ValidUntil;

        await database.SaveChangesAsync();

        return PreferredRouteDto.FromModel(route);
    }

    [HttpDelete("{id}")]
    [Authorize(Roles = UserRoles.EventCoordinator)]
    public async Task<PreferredRouteDto> DeletePreferredRoute(Ulid id)
    {
        var route = await database.PreferredRoute.FindAsync(id)
            ?? throw new ArgumentException("preferred route not found", nameof(id));

        database.PreferredRoute.Remove(route);
        await database.SaveChangesAsync();

        return PreferredRouteDto.FromModel(route);
    }
}
