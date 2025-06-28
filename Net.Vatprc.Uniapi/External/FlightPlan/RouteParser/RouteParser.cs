using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class RouteParser(string rawRoute, INavdataProvider navdata) : IParseContext
{
    protected readonly IList<ITokenHandler> TokenHandlers =
    [
        new AirportTokenHandler(),
        new SidTokenHandler(),
        new StarTokenHandler(),
        new AirwayTokenHandler(),
        new WaypointTokenHandler(),
    ];

    public INavdataProvider NavdataProvider => navdata;

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

    RouteToken IParseContext.CurrentSegment => CurrentSegment;

    public void MoveToNextSegment()
    {
        if (CurrentSegmentIndex + 1 < SegmentCount)
        {
            CurrentSegmentIndex++;
        }
    }

    public void MoveToPreviousSegment()
    {
        if (CurrentSegmentIndex > 0)
        {
            CurrentSegmentIndex--;
        }
    }

    public async Task ParseSegment(bool advance = true)
    {
        foreach (var handler in TokenHandlers)
        {
            if (handler.IsAllowed(this, NavdataProvider))
            {
                if (handler.NeedParseNextSegment)
                {
                    await TryParseNextSegment();
                }
                await handler.Resolve(this, NavdataProvider);
                break;
            }
        }
        if (advance)
        {
            MoveToNextSegment();
        }
    }

    public async Task TryParseNextSegment()
    {
        if (NextSegment == null) return;

        MoveToNextSegment();
        await ParseSegment(advance: false);
        MoveToPreviousSegment();
    }

    public async Task ParseAllSegments()
    {
        while (CurrentSegmentIndex < SegmentCount)
        {
            await ParseSegment();
        }
    }
}
