using System.Text;

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

    public class InvalidRouteException : Exception { }

    public static string NormalizeRoute(VATPRCContext db, string departure, string arrival, string route)
    {
        var sb = new StringBuilder();
        foreach (var segment in route.Split(' '))
        {
            if (sb.Length == 0)
            {
                db.Airport.First(x => x.Identifier == departure);
            }
            else
            {

            }
            sb.Append(' ');
        }
        return sb.ToString();
    }
}
