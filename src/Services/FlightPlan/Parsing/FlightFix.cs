using Arinc424;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class FlightFix
{
    public required string Id { get; set; }
    public required string Identifier { get; set; }
    public required FixType Type { get; set; }
    public required Geo? Geo { get; set; }

    public enum FixType
    {
        Airport,
        Waypoint,
        Vhf,
        Ndb,
        GeoCoord,
        Unknown,
    }
}
