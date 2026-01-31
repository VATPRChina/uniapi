namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class FlightLeg
{
    public required FlightFix From { get; set; }
    public required FlightFix To { get; set; }
    public required (Ulid, Ulid)? LegId { get; set; }
    public required string LegIdentifier { get; set; }
    public required LegType Type { get; set; }

    public enum LegType
    {
        Airway,
        Sid,
        Star,
        Direct,
    }
}
