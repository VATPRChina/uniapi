namespace Net.Vatprc.Uniapi.External.FlightPlan.Parsing;

public class FlightFix
{
    public required Ulid Id { get; set; }
    public required string Identifier { get; set; }
    public required FixType Type { get; set; }

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
