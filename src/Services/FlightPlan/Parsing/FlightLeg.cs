using Arinc424;
using Arinc424.Routing;
using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class FlightLeg
{
    public required FlightFix From { get; set; }
    public required FlightFix To { get; set; }
    public required (string, string)? LegId { get; set; }
    public required string LegIdentifier { get; set; }
    public required LegType Type { get; set; }
    public required (AirwayPoint, AirwayPoint)? Points { get; set; }
    public required Airway? Airway { get; set; }

    public enum LegType
    {
        Airway,
        Sid,
        Star,
        Direct,
    }
}
