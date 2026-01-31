using Arinc424;
using Arinc424.Ground;
using Arinc424.Navigation;
using Arinc424.Procedures;
using Arinc424.Waypoints;
using Net.Vatprc.Uniapi.Services.FlightPlan;
using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Adapters;

public class Arinc424NavdataAdapter : INavdataProvider
{
    protected const string Arinc424DataPath = "Data/cesfpl.pc";
    protected Data424 Data { get; set; } = null!;

    public bool ExistsAirwayWithFix(string ident, string fixIdent)
    {
        var exists = Data.Airways
            .Where(a => a.Identifier == ident)
            .Any(a => a.Sequence.Any(f => f.Fix.Identifier == fixIdent));
        return exists;
    }

    public Airport? FindAirport(string ident)
    {
        var airport = Data.Airports.SingleOrDefault(a => a.Identifier == ident);
        return airport;
    }

    public IEnumerable<AirwayLeg> FindAirwayLegs(string ident)
    {
        var airways = Data.Airways.Where(a => a.Identifier == ident).ToList();
        var legs = airways
            .SelectMany(airway => airway.Sequence.SkipLast(1)
                .Zip(airway.Sequence.Skip(1), (from, to) => new AirwayLeg
                {
                    Airway = airway,
                    From = from,
                    To = to,
                    Flipped = false,
                })
                .ToList());
        return legs;
    }

    public Nondirect? FindNdbNavaid(string ident, double lat, double lon)
    {
        var ndb = Data.Nondirects
            .Where(a => a.Identifier == ident)
            .OrderBy(w => Geography.DistanceBetweenPoints(w.Coordinates.Latitude, w.Coordinates.Longitude, lat, lon))
            .FirstOrDefault();
        return ndb;
    }

    public Departure? FindSid(string ident, string airportIdent)
    {
        var airport = Data.Airports.SingleOrDefault(a => a.Identifier == airportIdent);
        if (airport == null) return null;

        var sid = airport.Departures?.SingleOrDefault(p => p.Identifier == ident);
        return sid;
    }

    public Arrival? FindStar(string ident, string airportIdent)
    {
        var airport = Data.Airports.SingleOrDefault(a => a.Identifier == airportIdent);
        if (airport == null) return null;

        var star = airport.Arrivals?.SingleOrDefault(p => p.Identifier == ident);
        return star;
    }

    public Omnidirect? FindVhfNavaid(string ident, double lat, double lon)
    {
        var vhf = Data.Omnidirects
            .Where(a => a.Identifier == ident)
            .OrderBy(w => Geography.DistanceBetweenPoints(
                w.Coordinates.Latitude, w.Coordinates.Longitude,
                lat, lon))
            .FirstOrDefault();
        return vhf;
    }

    public Waypoint? FindWaypoint(string ident, double lat, double lon)
    {
        var waypoint = Data.EnrouteWaypoints
            .Where(a => a.Identifier == ident)
            .OrderBy(w => Geography.DistanceBetweenPoints(
                w.Coordinates.Latitude, w.Coordinates.Longitude,
                lat, lon))
            .FirstOrDefault();
        return waypoint;
    }

    public async Task InitializeAsync()
    {
        var meta = Meta424.Create(Supplement.V18);
        var strings = await File.ReadAllLinesAsync(Arinc424DataPath);
        var data = Data424.Create(meta, strings, out var invalid, out var skipped);
        Data = data;
    }
}
