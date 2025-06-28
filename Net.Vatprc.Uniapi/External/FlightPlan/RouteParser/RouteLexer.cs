using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;
using Serilog;

namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;

public class RouteLexer(string rawRoute, INavdataProvider navdata) : ILexerContext
{
    protected readonly Serilog.ILogger Logger = Log.ForContext<RouteLexer>();

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

    RouteToken ILexerContext.CurrentSegment => CurrentSegment;

    public void MoveToNextSegment()
    {
        Logger.Debug("Moving to next segment: {CurrentSegmentIndex} -> {NextSegmentIndex}",
            CurrentSegmentIndex, CurrentSegmentIndex + 1);
        if (CurrentSegmentIndex + 1 <= SegmentCount)
        {
            CurrentSegmentIndex++;
        }
    }

    public async Task ParseSegment(bool skipRequireNextSegment)
    {
        Logger.Debug("Parsing segment {Index}/{TotalSegmentCount}: {Segment}",
            CurrentSegmentIndex, SegmentCount, CurrentSegment.Value);
        foreach (var handler in TokenHandlers)
        {
            if (skipRequireNextSegment && handler.NeedParseNextSegment) { continue; }
            Logger.Verbose("Checking handler {Handler} for segment {Segment}",
                handler.GetType().Name, CurrentSegment.Value);
            if (handler.IsAllowed(this, NavdataProvider))
            {
                Logger.Verbose("Handler {Handler} is allowed for segment {Segment}",
                    handler.GetType().Name, CurrentSegment.Value);
                await handler.Resolve(this, NavdataProvider);
                Logger.Verbose("Handler {Handler} resolved segment {Segment} to kind {Kind} and id {Id}",
                    handler.GetType().Name, CurrentSegment.Value, CurrentSegment.Kind, CurrentSegment.Id);
                if (CurrentSegment.Kind != RouteTokenKind.UNKNOWN)
                {
                    break;
                }
                Logger.Verbose("Handler {Handler} did not resolve segment {Segment}, continuing to next handler",
                    handler.GetType().Name, CurrentSegment.Value);
            }
        }
        MoveToNextSegment();
    }

    public async Task ParseAllSegments()
    {
        // Pass 1
        while (CurrentSegmentIndex < SegmentCount)
        {
            await ParseSegment(skipRequireNextSegment: true);
        }
        // Pass 2
        CurrentSegmentIndex = 0;
        while (CurrentSegmentIndex < SegmentCount)
        {
            await ParseSegment(skipRequireNextSegment: false);
        }
    }

    public class NextSegmentLexerContextWrapper(ILexerContext context) : ILexerContext
    {
        public int CurrentSegmentIndex => context.CurrentSegmentIndex + 1;

        public int SegmentCount => context.SegmentCount;

        public RouteToken CurrentSegment
        {
            get
            {
                if (context.NextSegment != null)
                {
                    return context.NextSegment;
                }
                throw new InvalidOperationException("No next segment available.");
            }
        }

        public RouteToken? LastSegment => context.CurrentSegment;

        public RouteToken? NextSegment => null;

        public double CurrentLat { get => context.CurrentLat; set => context.CurrentLat = value; }

        public double CurrentLon { get => context.CurrentLon; set => context.CurrentLon = value; }
    }
}
