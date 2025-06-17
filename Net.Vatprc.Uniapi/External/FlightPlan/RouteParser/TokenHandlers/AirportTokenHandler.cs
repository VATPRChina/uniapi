namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

public class AirportTokenHandler : ITokenHandler
{
    public bool IsAllowed(IParseContext context, INavdataProvider navdataProvider)
    {
        return context.CurrentSegmentIndex == 0
            || context.CurrentSegmentIndex == context.SegmentCount - 1;
    }

    public async Task Resolve(IParseContext context, INavdataProvider navdataProvider)
    {
        var airport = await navdataProvider.FindAirport(context.CurrentSegment.Value);
        if (airport == null) return;

        context.CurrentSegment.Kind = RouteTokenKind.AIRPORT;
        context.CurrentSegment.Value = airport.Identifier;
        context.CurrentSegment.Id = airport.Id;
        context.CurrentLon = airport.Longitude;
        context.CurrentLat = airport.Latitude;
    }
}
