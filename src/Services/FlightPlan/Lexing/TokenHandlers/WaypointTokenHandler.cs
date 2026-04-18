using Net.Vatprc.Uniapi.Models.Navdata.Fixes;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class WaypointTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return true;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var fix = await navdataProvider.FindFix(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (fix == null) return false;

        context.CurrentSegment = new FixToken
        {
            Value = fix switch
            {
                Waypoint f => f.Identifier,
                VhfNavaid f => f.Identifier,
                NdbNavaid f => f.Identifier,
                _ => context.CurrentSegment.Value,
            },
            Fix = fix,
        };
        return true;
    }
}
