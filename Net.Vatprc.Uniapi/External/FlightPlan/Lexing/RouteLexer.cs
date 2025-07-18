using Net.Vatprc.Uniapi.External.FlightPlan.Lexing.TokenHandlers;
using Serilog;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Lexing;

public class RouteLexer(string rawRoute, INavdataProvider navdata) : ILexerContext
{
    protected readonly Serilog.ILogger Logger = Log.ForContext<RouteLexer>();

    protected readonly IList<ITokenHandler> TokenHandlers =
    [
        new AirportTokenHandler(),
        new SidTokenHandler(),
        new StarTokenHandler(),
        new AirwayTokenHandler(),
        new DctTokenHandler(),
        new WaypointTokenHandler(),
        new Geo7CharTokenHandler(),
        new Geo11CharTokenHandler(),
        new AirportFallbackTokenHandler(),
        new SidFallbackTokenHandler(),
        new StarFallbackTokenHandler(),
        new AirwayFallbackTokenHandler(),
    ];

    public INavdataProvider NavdataProvider => navdata;

    public IList<RouteToken> Tokens =
        rawRoute
            .Split(' ')
            .Select((segment, index) =>
            {
                // trim and remove anything after first '/'
                var slashIndex = segment.IndexOf('/');
                if (slashIndex >= 0)
                {
                    segment = segment[..slashIndex];
                }
                segment = segment.Trim();

                return new RouteToken
                {
                    Kind = RouteTokenKind.UNKNOWN,
                    Value = segment,
                    Id = Ulid.Empty,
                };
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

        CurrentSegment.Kind = RouteTokenKind.UNKNOWN;
        CurrentSegment.Id = Ulid.Empty;

        foreach (var handler in TokenHandlers)
        {
            if (skipRequireNextSegment && handler.NeedParseNextSegment) { continue; }
            Logger.Verbose("Checking handler {Handler} for segment {Segment}",
                handler.GetType().Name, CurrentSegment.Value);
            if (handler.IsAllowed(this, NavdataProvider))
            {
                Logger.Verbose("Handler {Handler} is allowed for segment {Segment}",
                    handler.GetType().Name, CurrentSegment.Value);
                var resolved = await handler.Resolve(this, NavdataProvider);
                Logger.Verbose("Handler {Handler} resolved segment {Segment} to kind {Kind} and id {Id}",
                    handler.GetType().Name, CurrentSegment.Value, CurrentSegment.Kind, CurrentSegment.Id);
                if (CurrentSegment.Kind != RouteTokenKind.UNKNOWN && resolved)
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
        CurrentLat = 0;
        CurrentLon = 0;
        while (CurrentSegmentIndex < SegmentCount)
        {
            await ParseSegment(skipRequireNextSegment: false);
        }

        Logger.Information("Route parsing completed. Total segments: {SegmentCount}", SegmentCount);
        if (Logger.IsEnabled(Serilog.Events.LogEventLevel.Debug))
        {
            foreach (var token in Tokens)
            {
                Logger.Debug("Segment {Index}: {Kind} - {Value} (Id: {Id})",
                    Tokens.IndexOf(token), token.Kind, token.Value, token.Id);
            }
        }
    }
}
