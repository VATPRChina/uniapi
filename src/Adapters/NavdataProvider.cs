using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Services.FlightPlan;

namespace Net.Vatprc.Uniapi.Adapters;

public class NavdataProvider(NavadataAdapter adapter) : INavdataProvider
{
    public async Task<bool> ExistsAirwayWithFix(string ident, string fixIdent)
    {
        var airways = await adapter.GetAirwayLegLookupAsync();
        return airways.TryGetValue(ident, out var legs)
            && legs.Any(leg =>
                string.Equals(leg.From.Identifier, fixIdent, StringComparison.OrdinalIgnoreCase)
                || string.Equals(leg.To.Identifier, fixIdent, StringComparison.OrdinalIgnoreCase));
    }

    public async Task<Airport?> FindAirport(string ident)
    {
        var airports = await adapter.GetAirportsAsync();
        return airports.TryGetValue(ident, out var airport) ? airport : null;
    }

    public async Task<Fix?> FindFix(string ident, double lat, double lon)
    {
        var candidates = new List<FixWithIdentifier>();

        var waypoints = await adapter.GetWaypointsAsync();
        if (waypoints.TryGetValue(ident, out var waypointMatches))
        {
            candidates.AddRange(waypointMatches);
        }

        var vhfNavaids = await adapter.GetVhfNavaidsAsync();
        if (vhfNavaids.TryGetValue(ident, out var vhfMatches))
        {
            candidates.AddRange(vhfMatches);
        }

        var ndbNavaids = await adapter.GetNdbNavaidsAsync();
        if (ndbNavaids.TryGetValue(ident, out var ndbMatches))
        {
            candidates.AddRange(ndbMatches);
        }

        if (candidates.Count == 0)
        {
            return null;
        }

        if (candidates.Count == 1)
        {
            return candidates[0];
        }

        return candidates
            .OrderBy(fix =>
            {
                var dLat = fix.Latitude - lat;
                var dLon = fix.Longitude - lon;
                return (dLat * dLat) + (dLon * dLon);
            })
            .First();
    }

    public async Task<Procedure?> FindSid(string ident, string airportIdent)
    {
        var procedures = await adapter.GetProceduresByAirportAsync('D');
        return procedures.TryGetValue(airportIdent, out var airportProcedures)
            && airportProcedures.TryGetValue(ident, out var procedure)
            ? procedure
            : null;
    }

    public async Task<Procedure?> FindStar(string ident, string airportIdent)
    {
        var procedures = await adapter.GetProceduresByAirportAsync('E');
        return procedures.TryGetValue(airportIdent, out var airportProcedures)
            && airportProcedures.TryGetValue(ident, out var procedure)
            ? procedure
            : null;
    }

    public async IAsyncEnumerable<AirwayLeg> GetAirwayLegs(string ident)
    {
        var airways = await adapter.GetAirwayLegLookupAsync();
        if (!airways.TryGetValue(ident, out var legs))
        {
            yield break;
        }

        foreach (var leg in legs)
        {
            yield return leg;
        }
    }

    public Task<IList<PreferredRoute>> GetRecommendedRoutes(string dep, string arr)
    {
        return adapter.GetPreferredRouteAsync(dep, arr);
    }
}
