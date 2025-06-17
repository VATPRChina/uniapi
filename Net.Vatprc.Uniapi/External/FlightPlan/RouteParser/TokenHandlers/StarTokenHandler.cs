namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

public class StarTokenHandler : ITokenHandler
{
    public bool NeedParseNextSegment => true;

    public bool IsAllowed(IParseContext context, INavdataProvider navdataProvider)
    {
        return context.NextSegment?.Kind == RouteTokenKind.AIRPORT;
    }

    public async Task Resolve(IParseContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return;
        var proc = await navdataProvider.FindStar(context.CurrentSegment.Value, context.LastSegment.Value);
        if (proc == null) return;

        context.CurrentSegment.Kind = RouteTokenKind.STAR;
        context.CurrentSegment.Id = proc.Id;
        context.CurrentSegment.Value = proc.Identifier;
    }
}
