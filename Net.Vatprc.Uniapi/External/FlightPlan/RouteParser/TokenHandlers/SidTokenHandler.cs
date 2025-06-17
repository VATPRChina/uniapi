namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

public class SidTokenHandler : ITokenHandler
{
    public bool IsAllowed(IParseContext context, INavdataProvider navdataProvider)
    {
        return context.LastSegment?.Kind == RouteTokenKind.AIRPORT;
    }

    public async Task Resolve(IParseContext context, INavdataProvider navdataProvider)
    {
        if (context.LastSegment == null) return;
        var proc = await navdataProvider.FindSid(context.CurrentSegment.Value, context.LastSegment.Value);
        if (proc == null) return;

        context.CurrentSegment.Kind = RouteTokenKind.SID;
        context.CurrentSegment.Id = proc.Id;
        context.CurrentSegment.Value = proc.Identifier;
    }
}
