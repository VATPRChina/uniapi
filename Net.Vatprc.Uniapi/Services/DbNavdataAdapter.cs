using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer;

public class DbNavdataAdapter(VATPRCContext dbContext) : INavdataProvider
{
    protected VATPRCContext DbContext => dbContext;

    public async Task<bool> ExistsAirwayWithFix(string ident, string fixIdent)
    {
        var fix = await DbContext.AirwayFix
            .Include(f => f.Airway)
            .Where(f => f.FixIdentifier == fixIdent && f.Airway!.Identifier == ident)
            .FirstOrDefaultAsync();
        return fix != null;
    }

    public async Task<Airport?> FindAirport(string ident)
    {
        var airport = await DbContext.Airport.FirstOrDefaultAsync(a => a.Identifier == ident);
        return airport;
    }

    public async Task<Procedure?> FindSid(string ident, string airportIdent)
    {
        var procedure = await DbContext.Procedure
            .Include(p => p.Airport)
            .Where(p => p.Identifier == ident && p.Airport!.Identifier == airportIdent && p.SubsectionCode == 'D')
            .FirstOrDefaultAsync();
        return procedure;
    }

    public async Task<Procedure?> FindStar(string ident, string airportIdent)
    {
        var procedure = await DbContext.Procedure
            .Include(p => p.Airport)
            .Where(p => p.Identifier == ident && p.Airport!.Identifier == airportIdent && p.SubsectionCode == 'E')
            .FirstOrDefaultAsync();
        return procedure;
    }

    public async Task<VhfNavaid?> FindVhfNavaid(string ident, double lat, double lon)
    {
        var vhf = await DbContext.VhfNavaid.Where(a => a.VorIdentifier == ident || a.DmeIdentifier == ident)
            .AsAsyncEnumerable()
            .OrderBy(w => Geography.DistanceBetweenPoints(
                w.VorLatitude ?? w.DmeLatitude ?? 0.0,
                w.VorLongitude ?? w.DmeLongitude ?? 0.0, lat, lon))
            .FirstOrDefaultAsync();
        return vhf;
    }

    public async Task<NdbNavaid?> FindNdbNavaid(string ident, double lat, double lon)
    {
        var ndb = await DbContext.NdbNavaid.Where(a => a.Identifier == ident)
            .AsAsyncEnumerable()
            .OrderBy(w => Geography.DistanceBetweenPoints(w.Latitude, w.Longitude, lat, lon))
            .FirstOrDefaultAsync();
        return ndb;
    }

    public async Task<Waypoint?> FindWaypoint(string ident, double lat, double lon)
    {
        var waypoint = await DbContext.Waypoint.Where(a => a.Identifier == ident)
            .AsAsyncEnumerable()
            .OrderBy(w => Geography.DistanceBetweenPoints(w.Latitude, w.Longitude, lat, lon))
            .FirstOrDefaultAsync();
        return waypoint;
    }

    public async IAsyncEnumerable<INavdataProvider.AirwayLeg> FindAirwayLegs(string ident)
    {
        var airways = DbContext.Airway.Where(a => a.Identifier == ident)
            .Include(a => a.Fixes)
            .ToAsyncEnumerable();
        var legs = new List<INavdataProvider.AirwayLeg>();

        await foreach (var airway in airways)
        {
            airway.Fixes = airway.Fixes.OrderBy(f => f.SequenceNumber).ToList();
            for (int i = 0; i < airway.Fixes.Count - 1; i++)
            {
                var fromFix = airway.Fixes[i];
                var toFix = airway.Fixes[i + 1];
                yield return new INavdataProvider.AirwayLeg
                {
                    Ident = airway.Identifier,
                    FromFixIcaoCode = fromFix.FixIcaoCode,
                    FromFixIdentifier = fromFix.FixIdentifier,
                    FromFixId = fromFix.Id,
                    FromFixType = INavdataProvider.FixType.Waypoint, // TODO: add real fix type if available
                    ToFixIcaoCode = toFix.FixIcaoCode,
                    ToFixIdentifier = toFix.FixIdentifier,
                    ToFixId = toFix.Id,
                    ToFixType = INavdataProvider.FixType.Waypoint, // TODO: add real fix type if available
                };
            }
        }
    }

    public Task<AirwayFix?> GetAirwayFix(Ulid id)
    {
        return DbContext.AirwayFix
            .Include(f => f.Airway)
            .FirstOrDefaultAsync(f => f.Id == id);
    }

    public async Task<IList<string>> GetRecommendedRoutes(string dep, string arr)
    {
        var recommendedRoute = await DbContext.PreferredRoute
            .Where(r => r.Departure == dep && r.Arrival == arr)
            .Select(r => r.RawRoute)
            .ToArrayAsync();

        return recommendedRoute;
    }
}
