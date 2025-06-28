namespace Net.Vatprc.Uniapi.External.FlightPlan.RouteParser.TokenHandlers;

public class WaypointTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return true;
    }

    public async Task Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var vhf = await navdataProvider.FindVhfNavaid(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (vhf != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.VHF;
            context.CurrentSegment.Id = vhf.Id;
            context.CurrentLon = vhf.VorLongitude ?? vhf.DmeLongitude
                ?? throw new InvalidOperationException("VHF must have geographic coordinates");
            context.CurrentLat = vhf.VorLatitude ?? vhf.DmeLatitude
                ?? throw new InvalidOperationException("VHF must have geographic coordinates");
            return;
        }
        var ndb = await navdataProvider.FindNdbNavaid(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (ndb != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.NDB;
            context.CurrentSegment.Id = ndb.Id;
            context.CurrentLon = ndb.Longitude;
            context.CurrentLat = ndb.Latitude;
            return;
        }
        var waypoint = await navdataProvider.FindWaypoint(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (waypoint != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.WAYPOINT;
            context.CurrentSegment.Id = waypoint.Id;
            context.CurrentLon = waypoint.Longitude;
            context.CurrentLat = waypoint.Latitude;
            return;
        }
    }
}
