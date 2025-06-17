namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public interface IParseContext
{
    public int CurrentSegmentIndex { get; }

    public int SegmentCount { get; }

    public RouteToken CurrentSegment { get; set; }

    public RouteToken? LastSegment { get; }

    public RouteToken? NextSegment { get; }

    public double CurrentLat { get; set; }

    public double CurrentLon { get; set; }
}
