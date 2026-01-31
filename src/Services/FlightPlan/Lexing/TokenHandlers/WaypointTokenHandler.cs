using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing.TokenHandlers;

public class WaypointTokenHandler : ITokenHandler
{
    public bool IsAllowed(ILexerContext context, INavdataProvider navdataProvider)
    {
        return true;
    }

    public async Task<bool> Resolve(ILexerContext context, INavdataProvider navdataProvider)
    {
        var vhf = navdataProvider.FindVhfNavaid(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (vhf != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.VHF;
            context.CurrentSegment.Id = vhf.RecordId;
            context.CurrentSegment.Geo = vhf;
            context.CurrentLon = vhf.Coordinates.Longitude;
            context.CurrentLat = vhf.Coordinates.Latitude;
            return true;
        }
        var ndb = navdataProvider.FindNdbNavaid(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (ndb != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.NDB;
            context.CurrentSegment.Id = ndb.RecordId;
            context.CurrentSegment.Geo = ndb;
            context.CurrentLon = ndb.Coordinates.Longitude;
            context.CurrentLat = ndb.Coordinates.Latitude;
            return true;
        }
        var waypoint = navdataProvider.FindWaypoint(context.CurrentSegment.Value, context.CurrentLat, context.CurrentLon);
        if (waypoint != null)
        {
            context.CurrentSegment.Kind = RouteTokenKind.WAYPOINT;
            context.CurrentSegment.Id = waypoint.RecordId;
            context.CurrentSegment.Geo = waypoint;
            context.CurrentLon = waypoint.Coordinates.Longitude;
            context.CurrentLat = waypoint.Coordinates.Latitude;
            return true;
        }
        return false;
    }
}
