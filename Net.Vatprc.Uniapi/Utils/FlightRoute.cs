using System.Collections.Immutable;

namespace Net.Vatprc.Uniapi.Utils;

public static partial class FlightRoute
{
    public static string SimplifyRoute(string route)
    {
        IList<string> segments = [];
        foreach (var segment in route.Split(' '))
        {
            var normalizedSegment = segment.ToUpperInvariant();
            if (segment.Contains('/')) normalizedSegment = segment.Split('/')[0];

            if (segments.Count >= 3 && segments[^3] == segments[^1])
            {
                segments[^2] = normalizedSegment;
                segments.RemoveAt(segments.Count - 1);
            }
            else segments.Add(segment);
        }
        return string.Join(' ', segments);
    }

    internal record class RouteToken(RouteToken.Kinds Kind, string Ident, Ulid Id, string? IcaoCode = null)
    {
        public enum Kinds
        {
            Sid,
            Star,
            Waypoint,
            Airway,
        }
    }

    public class InvalidRouteException(string message) : Exception(message) { }

    internal static async Task<RouteToken?> FindWaypoint(this VATPRCContext db, double lat, double lon, string ident)
    {
        var waypoint = await db.Waypoint.Where(a => a.Identifier == ident).AsAsyncEnumerable().OrderBy(w => Geography.DistanceBetweenPoints(w.Latitude, w.Longitude, lat, lon)).FirstOrDefaultAsync();
        if (waypoint != null)
        {
            return new RouteToken(RouteToken.Kinds.Waypoint, waypoint.Identifier, waypoint.Id, waypoint.IcaoCode);
        }

        var vhf = await db.VhfNavaid.Where(a => a.VorIdentifier == ident || a.DmeIdentifier == ident).AsAsyncEnumerable().OrderBy(w => Geography.DistanceBetweenPoints(w.VorLatitude ?? w.DmeLatitude ?? 0.0, w.VorLongitude ?? w.DmeLongitude ?? 0.0, lat, lon)).FirstOrDefaultAsync();
        if (vhf != null)
        {
            return new RouteToken(RouteToken.Kinds.Waypoint, ident, vhf.Id, vhf.IcaoCode);
        }

        var ndb = await db.NdbNavaid.Where(a => a.Identifier == ident).AsAsyncEnumerable().OrderBy(w => Geography.DistanceBetweenPoints(w.Latitude, w.Longitude, lat, lon)).FirstOrDefaultAsync();
        if (ndb != null)
        {
            return new RouteToken(RouteToken.Kinds.Waypoint, ident, ndb.Id, ndb.IcaoCode);
        }
        return new RouteToken(RouteToken.Kinds.Waypoint, ident, Ulid.Empty);
    }

    public static async Task<string> NormalizeRoute(VATPRCContext db, string departure, string arrival, string route)
    {
        var airport = await db.Airport.FirstOrDefaultAsync(a => a.Identifier == departure)
            ?? throw new InvalidRouteException($"Airport {departure} not found.");
        var (lat, lon) = (airport.Latitude, airport.Longitude);

        var sb = new List<RouteToken>();
        var segments = route.Split(' ').ToImmutableArray();
        for (int i = 0; i < segments.Length; i++)
        {
            var segment = segments[i].Split('/').FirstOrDefault() ?? string.Empty;
            if (i == 0)
            {
                var sid = await db.Procedure.FirstOrDefaultAsync(p => p.Airport!.Identifier == departure && p.SubsectionCode == 'D' && p.Identifier == segment);
                if (sid != null)
                {
                    sb.Add(new RouteToken(RouteToken.Kinds.Sid, segment, sid.Id));
                    continue;
                }
                else
                {
                    sb.Add(new RouteToken(RouteToken.Kinds.Sid, "SID", Ulid.Empty));
                    // pass to waypoint handling
                }
            }
            else if (i == segments.Length - 1)
            {
                var star = await db.Procedure.FirstOrDefaultAsync(p => p.Airport!.Identifier == arrival && p.SubsectionCode == 'E' && p.Identifier == segment);
                if (star != null)
                {
                    sb.Add(new RouteToken(RouteToken.Kinds.Star, segment, star.Id));
                    continue;
                }
                // else, pass to waypoint handling
            }

            if (sb.Count >= 2 && sb[^2].Kind == RouteToken.Kinds.Waypoint && sb[^1].Kind == RouteToken.Kinds.Airway)
            {
                var airwayId = sb[^1].Id;
                var airwayIdent = sb[^1].Ident;
                var airwayFixes = await db.AirwayFix.OrderBy(f => f.SequenceNumber).Where(f => f.AirwayId == airwayId).ToListAsync();
                var from = airwayFixes.FirstOrDefault(f => f.FixIdentifier == sb[^2].Ident)
                    ?? throw new InvalidOperationException($"From fix {segment} is not on airway {airwayIdent}.");
                var to = airwayFixes.FirstOrDefault(f => f.FixIdentifier == segment)
                    ?? throw new InvalidRouteException($"To fix {segment} is not on airway {airwayIdent}.");
                var fromIndex = airwayFixes.IndexOf(from);
                var toIndex = airwayFixes.IndexOf(to);
                if (fromIndex <= toIndex)
                {
                    for (var j = fromIndex + 1; j < toIndex; j++)
                    {
                        sb.Add(new RouteToken(RouteToken.Kinds.Waypoint, airwayFixes[j].FixIdentifier, Ulid.Empty, airwayFixes[j].FixIcaoCode));
                        sb.Add(new RouteToken(RouteToken.Kinds.Airway, airwayIdent, airwayId));
                    }
                }
                else
                {
                    for (var j = fromIndex - 1; j > toIndex; j--)
                    {
                        sb.Add(new RouteToken(RouteToken.Kinds.Waypoint, airwayFixes[j].FixIdentifier, Ulid.Empty, airwayFixes[j].FixIcaoCode));
                        sb.Add(new RouteToken(RouteToken.Kinds.Airway, airwayIdent, airwayId));
                    }
                }
                sb.Add(new RouteToken(RouteToken.Kinds.Waypoint, segment, Ulid.Empty, to.FixIcaoCode));
            }
            else if (sb.Count >= 1 && sb[^1].Kind == RouteToken.Kinds.Waypoint)
            {
                var (lastIdent, lastIcaoCode) = (sb[^1].Ident, sb[^1].IcaoCode);
                var fromFix = await db.AirwayFix.FirstOrDefaultAsync(f => f.FixIdentifier == lastIdent && f.FixIcaoCode == lastIcaoCode && f.Airway!.Identifier == segment);
                if (fromFix != null)
                {
                    sb.Add(new RouteToken(RouteToken.Kinds.Airway, segment, fromFix.AirwayId));
                }
                else
                {
                    var token = await FindWaypoint(db, lat, lon, segment) ??
                        throw new InvalidRouteException($"Waypoint or airway {segment} not found.");
                    sb.Add(new RouteToken(RouteToken.Kinds.Airway, "DCT", Ulid.Empty));
                    sb.Add(token);
                }
            }
            else if (sb[^1].Kind == RouteToken.Kinds.Sid)
            {
                var token = await FindWaypoint(db, lat, lon, segment) ??
                    throw new InvalidRouteException($"Waypoint {segment} not found.");
                sb.Add(token);
            }
            else
            {
                throw new InvalidRouteException("Unkown pattern");
            }

            if (i == segments.Length - 1)
            {
                sb.Add(new RouteToken(RouteToken.Kinds.Star, "STAR", Ulid.Empty));
            }
        }
        return string.Join(' ', sb.Select(x => x.Ident));
    }

    public static async Task<string> TryNormalizeRoute(VATPRCContext db, string departure, string arrival, string route)
    {
        try { return await NormalizeRoute(db, departure, arrival, route); }
        catch (Exception e) { return "ERROR: " + e.Message; }
    }
}
