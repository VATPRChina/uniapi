using Arinc424;
using Net.Vatprc.Uniapi.External.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Lexing;

public class Arinc424NavdataAdapter : INavdataProvider
{
    protected const string Arinc424DataPath = "Data/cesfpl.pc";
    protected Data424 Data { get; set; } = null!;

    public async Task InitializeAsync()
    {
        var meta = Meta424.Create(Supplement.V18);
        var strings = await File.ReadAllLinesAsync(Arinc424DataPath);
        var data = Data424.Create(meta, strings, out var invalid, out var skipped);
        Data = data;
    }

    public Task<bool> ExistsAirwayWithFix(string ident, string fixIdent)
    {
        var airway = Data.Airways.FirstOrDefault(a => a.Identifier == ident);
        if (airway == null) return Task.FromResult(false);
        var exists = airway.Sequence.Any(f => f.Fix.Identifier == fixIdent);
        return Task.FromResult(exists);
    }

    public Task<Airport?> FindAirport(string ident)
    {
        var airport = Data.Airports.FirstOrDefault(a => a.Identifier == ident);
        if (airport == null) return Task.FromResult<Airport?>(null);
        return Task.FromResult<Airport?>(new Airport
        {
            Id = Ulid.Empty, // TODO: $"{airport.Date}/{airport.Number}",
            Identifier = airport.Identifier,
            Latitude = airport.Coordinates.Latitude,
            Longitude = airport.Coordinates.Longitude,
            Elevation = airport.Elevation,
        });
    }

    public IAsyncEnumerable<INavdataProvider.AirwayLeg> FindAirwayLegs(string ident)
    {
        throw new NotImplementedException();
    }

    public Task<NdbNavaid?> FindNdbNavaid(string ident, double lat, double lon)
    {
        var ndb = Data.Nondirectionals
            .Where(n => n.Identifier == ident)
            .OrderBy(n => Geography.DistanceBetweenPoints(n.Coordinates.Latitude, n.Coordinates.Longitude, lat, lon))
            .FirstOrDefault();
        if (ndb == null) return Task.FromResult<NdbNavaid?>(null);
        return Task.FromResult<NdbNavaid?>(new NdbNavaid
        {
            Id = Ulid.Empty, // TODO: $"{ndb.Date}/{ndb.Number}",
            Identifier = ndb.Identifier,
            Latitude = ndb.Coordinates.Latitude,
            Longitude = ndb.Coordinates.Longitude,
        });
    }

    public Task<Procedure?> FindSid(string ident, string airportIdent)
    {
        throw new NotImplementedException();
    }

    public Task<Procedure?> FindStar(string ident, string airportIdent)
    {
        throw new NotImplementedException();
    }

    public Task<VhfNavaid?> FindVhfNavaid(string ident, double lat, double lon)
    {
        throw new NotImplementedException();
    }

    public Task<Waypoint?> FindWaypoint(string ident, double lat, double lon)
    {
        throw new NotImplementedException();
    }

    public Task<AirwayFix?> GetAirwayFix(Ulid id)
    {
        throw new NotImplementedException();
    }

    public Task<IList<PreferredRoute>> GetRecommendedRoutes(string dep, string arr)
    {
        throw new NotImplementedException();
    }

    public Task<string?> GetFullQualifiedFixIdentifier(Ulid id, INavdataProvider.FixType type)
    {
        throw new NotImplementedException();
    }
}
