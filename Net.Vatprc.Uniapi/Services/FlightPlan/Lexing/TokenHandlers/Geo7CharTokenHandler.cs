namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class Geo7CharTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegment.Value.Length == 7
            && (context.CurrentSegment.Value[2] == 'N' || context.CurrentSegment.Value[2] == 'S')
            && (context.CurrentSegment.Value[6] == 'E' || context.CurrentSegment.Value[6] == 'W');
    }

    public Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var latStr = context.CurrentSegment.Value[..2];
        if (!int.TryParse(latStr, out var lat))
        {
            throw new InvalidOperationException("Invalid latitude in geo coordinate");
        }
        var latSign = context.CurrentSegment.Value[2] == 'N' ? 1
            : context.CurrentSegment.Value[2] == 'S' ? -1
            : throw new InvalidOperationException("Invalid latitude sign in geo coordinate");

        var lonStr = context.CurrentSegment.Value[3..6];
        if (!int.TryParse(lonStr, out var lon))
        {
            throw new InvalidOperationException("Invalid longitude in geo coordinate");
        }
        var lonSign = context.CurrentSegment.Value[6] == 'E' ? 1
            : context.CurrentSegment.Value[6] == 'W' ? -1
            : throw new InvalidOperationException("Invalid longitude sign in geo coordinate");

        context.CurrentSegment.Kind = RouteTokenKind.GEO_COORD;
        context.CurrentSegment.Id = Ulid.Empty;
        context.CurrentLat = lat * latSign;
        context.CurrentLon = lon * lonSign;
        return Task.FromResult(true);
    }
}
