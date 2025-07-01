namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class FlightFix
{
    public Ulid ToNextLegAirwayId { get; set; } = Ulid.Empty;
    public string FixIdentifier { get; set; } = string.Empty;
    public Ulid FixId { get; set; } = Ulid.Empty;
    public bool IsUnknown { get; set; } = false;
}
