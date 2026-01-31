using Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

public class RouteLexer(string rawRoute, INavdataProvider navdata, ILogger<RouteLexer> Logger, ILoggerFactory loggerFactory) : ILexerContext
{
    protected readonly IList<ITokenHandler> TokenHandlers =
    [
        new InitialSpeedAndAltitudeTokenHandler(),
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
                    Id = string.Empty,
                    Geo = null,
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
        Logger.LogDebug("Moving to next segment: {CurrentSegmentIndex} -> {NextSegmentIndex}",
            CurrentSegmentIndex, CurrentSegmentIndex + 1);
        if (CurrentSegmentIndex + 1 <= SegmentCount)
        {
            CurrentSegmentIndex++;
        }
    }

    public async Task ParseSegment(bool skipRequireNextSegment)
    {
        Logger.LogDebug("Parsing segment {Index}/{TotalSegmentCount}: {Segment}",
            CurrentSegmentIndex, SegmentCount, CurrentSegment.Value);

        CurrentSegment.Kind = RouteTokenKind.UNKNOWN;
        CurrentSegment.Id = string.Empty;

        foreach (var handler in TokenHandlers)
        {
            if (skipRequireNextSegment && handler.NeedParseNextSegment) { continue; }
            Logger.LogTrace("Checking handler {Handler} for segment {Segment}",
                handler.GetType().Name, CurrentSegment.Value);
            if (handler.IsAllowed(this, NavdataProvider))
            {
                Logger.LogTrace("Handler {Handler} is allowed for segment {Segment}",
                    handler.GetType().Name, CurrentSegment.Value);
                var resolved = await handler.Resolve(this, NavdataProvider);
                Logger.LogTrace("Handler {Handler} resolved segment {Segment} to kind {Kind} and id {Id}",
                    handler.GetType().Name, CurrentSegment.Value, CurrentSegment.Kind, CurrentSegment.Id);
                if (CurrentSegment.Kind != RouteTokenKind.UNKNOWN && resolved)
                {
                    break;
                }
                Logger.LogTrace("Handler {Handler} did not resolve segment {Segment}, continuing to next handler",
                    handler.GetType().Name, CurrentSegment.Value);
            }
        }
        MoveToNextSegment();
    }

    public async Task ParseAllSegments(CancellationToken ct = default)
    {
        // Pass 1
        while (CurrentSegmentIndex < SegmentCount)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Parsing cancelled.");
                return;
            }
            await ParseSegment(skipRequireNextSegment: true);
        }
        // Pass 2
        CurrentSegmentIndex = 0;
        CurrentLat = 0;
        CurrentLon = 0;
        while (CurrentSegmentIndex < SegmentCount)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Parsing cancelled.");
                return;
            }
            await ParseSegment(skipRequireNextSegment: false);
        }

        Logger.LogInformation("Route parsing completed. Total segments: {SegmentCount}", SegmentCount);
        if (Logger.IsEnabled(LogLevel.Debug))
        {
            foreach (var token in Tokens)
            {
                Logger.LogDebug("Segment {Index}: {Kind} - {Value} (Id: {Id})",
                    Tokens.IndexOf(token), token.Kind, token.Value, token.Id);
            }
        }
    }

    public ILogger<T> GetLogger<T>()
    {
        return loggerFactory.CreateLogger<T>();
    }
}
