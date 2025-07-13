namespace Net.Vatprc.Uniapi.External.FlightPlan.Lexing;

public interface ILexerContext
{
    public int CurrentSegmentIndex { get; }

    public int SegmentCount { get; }

    public RouteToken CurrentSegment { get; }

    public RouteToken? LastSegment { get; }

    public RouteToken? NextSegment { get; }

    public double CurrentLat { get; set; }

    public double CurrentLon { get; set; }
}
