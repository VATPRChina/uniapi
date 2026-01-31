using Arinc424.Routing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

public record AirwayLeg
{
    public required Airway Airway { get; init; }
    public required AirwayPoint From { get; init; }
    public required AirwayPoint To { get; init; }
    public required bool Flipped { get; init; }
}
