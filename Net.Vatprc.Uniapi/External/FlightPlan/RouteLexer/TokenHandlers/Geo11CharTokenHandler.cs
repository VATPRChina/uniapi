namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteLexer.TokenHandlers;

public class Geo11CharTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegment.Value.Length == 11
            && (context.CurrentSegment.Value[4] == 'N' || context.CurrentSegment.Value[4] == 'S')
            && (context.CurrentSegment.Value[10] == 'E' || context.CurrentSegment.Value[10] == 'W');
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var latIntStr = context.CurrentSegment.Value[..2];
        if (!int.TryParse(latIntStr, out var latInt))
        {
            throw new InvalidOperationException("Invalid latitude in geo coordinate");
        }
        var latMinStr = context.CurrentSegment.Value[2..4];
        if (!int.TryParse(latMinStr, out var latMin))
        {
            throw new InvalidOperationException("Invalid latitude minutes in geo coordinate");
        }
        var lat = latInt + (latMin / 60.0);
        var latSign = context.CurrentSegment.Value[4] == 'N' ? 1
            : context.CurrentSegment.Value[4] == 'S' ? -1
            : throw new InvalidOperationException("Invalid latitude sign in geo coordinate");

        var lonIntStr = context.CurrentSegment.Value[5..8];
        if (!int.TryParse(lonIntStr, out var lonInt))
        {
            throw new InvalidOperationException("Invalid longitude in geo coordinate");
        }
        var lonMinStr = context.CurrentSegment.Value[8..10];
        if (!int.TryParse(lonMinStr, out var lonMin))
        {
            throw new InvalidOperationException("Invalid longitude minutes in geo coordinate");
        }
        var lon = lonInt + (lonMin / 60.0);
        var lonSign = context.CurrentSegment.Value[10] == 'E' ? 1
            : context.CurrentSegment.Value[10] == 'W' ? -1
            : throw new InvalidOperationException("Invalid longitude sign in geo coordinate");

        context.CurrentSegment.Kind = RouteTokenKind.GEO_COORD;
        context.CurrentSegment.Id = Ulid.Empty;
        context.CurrentLat = lat * latSign;
        context.CurrentLon = lon * lonSign;
        return Task.FromResult(true);
    }
}
