using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class DbNavdataAdapter(VATPRCContext dbContext) : INavdataProvider
{
    protected VATPRCContext DbContext => dbContext;

    public async Task<bool> ExistsAirwayWithFix(string ident, string fixIdent)
    {
        var fix = await DbContext.AirwayFix
            .Include(f => f.Airway)
            .Where(f => f.FixIdentifier == fixIdent && f.AirwayIdentifier == ident)
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
            .Where(p => p.Identifier == ident && p.AirportIdentifier == airportIdent && p.SubsectionCode == 'D')
            .FirstOrDefaultAsync();
        return procedure;
    }

    public async Task<Procedure?> FindStar(string ident, string airportIdent)
    {
        var procedure = await DbContext.Procedure
            .Where(p => p.Identifier == ident && p.AirportIdentifier == airportIdent && p.SubsectionCode == 'E')
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
}
