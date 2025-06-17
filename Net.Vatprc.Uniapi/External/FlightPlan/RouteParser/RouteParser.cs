namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class RouteParser(VATPRCContext db, string rawRoute) : IParseContext
{
    public VATPRCContext Db = db;

    public IList<RouteToken> Tokens =
        rawRoute
            .Split(' ')
            .Select((segment, index) => new RouteToken
            {
                Kind = RouteTokenKind.UNKNOWN,
                Value = segment,
                Id = Ulid.Empty,
            })
            .ToArray();

    public double CurrentLon { get; set; }

    public double CurrentLat { get; set; }

    public int CurrentSegmentIndex { get; protected set; } = 0;

    public int SegmentCount => Tokens.Count;

    public RouteToken CurrentSegment => Tokens[CurrentSegmentIndex];

    public RouteToken? LastSegment => CurrentSegmentIndex > 0
        ? Tokens[CurrentSegmentIndex - 1]
        : null;

    public RouteToken? NextSegment => CurrentSegmentIndex + 1 < SegmentCount
        ? Tokens[CurrentSegmentIndex + 1]
        : null;

    RouteToken IParseContext.CurrentSegment { get => CurrentSegment; set => throw new NotImplementedException(); }

    public void MoveToNextSegment()
    {
        if (CurrentSegmentIndex + 1 < SegmentCount)
        {
            CurrentSegmentIndex++;
        }
    }
}
