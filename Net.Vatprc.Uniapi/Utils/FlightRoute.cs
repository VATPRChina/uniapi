using System.Text.RegularExpressions;

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

    // public class Leg(string from, string to, string airway)
    // {
    //     public string From { get; set; } = from;
    //     public string To { get; set; } = to;
    //     public string Airway { get; set; } = airway;
    // }

    // public static IEnumerable<Leg> ParseLegs(string route)
    // {
    //     List<Leg> legs = [];
    //     foreach (var item in route.Split(' '))
    //     {
    //         if (legs.Count == 0)
    //         {
    //             if (IsSpeedOrAltitude(item)) continue;
    //             if (navdata.GetNavdataType(item) == INavdataIdentResolver.NavdataType.Sid)
    //             {
    //                 legs.Add(new Leg(from: string.Empty, to: string.Empty, airway: item));
    //                 continue;
    //             }
    //         }
    //         switch (navdata.GetNavdataType(item))
    //         {
    //             case INavdataIdentResolver.NavdataType.SignificantPoint:
    //                 if (!string.IsNullOrEmpty(legs[^1].From) && !string.IsNullOrEmpty(legs[^1].Airway) && string.IsNullOrEmpty(legs[^1].To))
    //                 {
    //                     legs[^1].To = item;
    //                 }
    //                 else
    //                 {
    //                     legs.Add(new Leg(from: item, to: string.Empty, airway: string.Empty));
    //                 }
    //                 break;
    //             case INavdataIdentResolver.NavdataType.AtsRoute:
    //                 legs[^1].Airway = item;
    //                 break;
    //             case INavdataIdentResolver.NavdataType.Star:
    //                 legs.Add(new Leg(from: string.Empty, to: string.Empty, airway: item));
    //                 break;
    //             case INavdataIdentResolver.NavdataType.NotFound:
    //                 throw new NotImplementedException();
    //         }
    //     }
    //     throw new NotImplementedException();
    // }

    // public static bool IsSpeedOrAltitude(string item)
    // {
    //     if (string.IsNullOrEmpty(item)) return false;
    //     return SpeedOrAltitudeIdent().IsMatch(item.ToUpperInvariant());
    // }

    // /// <summary>
    // /// Speed or altitude identifier. See https://wiki.ivao.aero/en/home/training/main/documentation/Flightplan.
    // /// </summary>
    // /// <returns></returns>
    // [GeneratedRegex(@"^((N|K)\d{4}|M\d{3})?((F|A)\d{3}|(S|M)\d{4}|VFR)?(PLUS)?$")]
    // private static partial Regex SpeedOrAltitudeIdent();

    // /// <summary>
    // /// ATS route identifier. See https://wiki.ivao.aero/en/home/training/documentation/Identification_of_ATS_routes.
    // /// </summary>
    // /// <returns></returns>
    // [GeneratedRegex(@"^[A-Z]?[A-Z]\d{1,4}[A-Z]?$")]
    // private static partial Regex AirwayIdent();

    // /// <summary>
    // /// SID/STAR identifier.
    // /// </summary>
    // /// <returns></returns>
    // [GeneratedRegex(@"^[A-Z]{1,}\d{1,}[A-Z]{1,}(xAR[A-Z]?)?$")]
    // private static partial Regex SidOrStarIdent();
}
