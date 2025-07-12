using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Utils;

public static class AltitudeHelper
{
    public static readonly IDictionary<int, int> StandardAltitudesToFlightLevel = new Dictionary<int, int>
    {
        {  300,  1000},
        {  900,  3000},
        { 1500,  4900},
        { 2100,  6900},
        { 2700,  8900},
        { 3300, 10800},
        { 3900, 12800},
        { 4500, 14800},
        { 5100, 16700},
        { 5700, 18700},
        { 6300, 20700},
        { 6900, 22600},
        { 7500, 24600},
        { 8100, 26600},
        { 8900, 29100},
        { 9500, 31100},
        {10100, 33100},
        {10700, 35100},
        {11300, 37100},
        {11900, 39100},
        {12500, 41100},
        {13700, 44900},
        {14900, 48900},

        {  600,  2000},
        { 1200,  3900},
        { 1800,  5900},
        { 2400,  7900},
        { 3000,  9800},
        { 3600, 11800},
        { 4200, 13800},
        { 4800, 15700},
        { 5400, 17700},
        { 6000, 19700},
        { 6600, 21700},
        { 7200, 23600},
        { 7800, 25600},
        { 8400, 27600},
        { 9200, 30100},
        { 9800, 32100},
        {10400, 34100},
        {11000, 36100},
        {11600, 38100},
        {12200, 40100},
        {13100, 43000},
        {14300, 46900},
    };
    public static readonly IDictionary<int, int> StandardAltitudesFromFlightLevel =
        StandardAltitudesToFlightLevel.ToDictionary(kvp => kvp.Value, kvp => kvp.Key);

    public static PreferredRoute.LevelRestrictionType GetLevelRestrictionTypeFromCruisingLevel(int cruisingLevel)
    {
        if (!StandardAltitudesFromFlightLevel.TryGetValue(cruisingLevel, out int metricLevel))
        {
            if (cruisingLevel % 1000 == 0)
            {
                if (cruisingLevel % 2000 == 0)
                {
                    return PreferredRoute.LevelRestrictionType.FlightLevelEven;
                }
                else
                {
                    return PreferredRoute.LevelRestrictionType.FlightLevelOdd;
                }
            }
            return PreferredRoute.LevelRestrictionType.Standard;
        }

        if (metricLevel % 200 == 0)
        {
            return PreferredRoute.LevelRestrictionType.StandardEven;
        }
        else
        {
            return PreferredRoute.LevelRestrictionType.StandardOdd;
        }
    }

    public static bool IsFlightLevelTypeMatching(
        PreferredRoute.LevelRestrictionType actual,
        PreferredRoute.LevelRestrictionType expected)
    {
        return expected switch
        {
            PreferredRoute.LevelRestrictionType.StandardEven => actual == PreferredRoute.LevelRestrictionType.StandardEven,
            PreferredRoute.LevelRestrictionType.StandardOdd => actual == PreferredRoute.LevelRestrictionType.StandardOdd,
            PreferredRoute.LevelRestrictionType.Standard => actual == PreferredRoute.LevelRestrictionType.Standard ||
                                                            actual == PreferredRoute.LevelRestrictionType.StandardEven ||
                                                            actual == PreferredRoute.LevelRestrictionType.StandardOdd,
            PreferredRoute.LevelRestrictionType.FlightLevelEven => actual == PreferredRoute.LevelRestrictionType.FlightLevelEven,
            PreferredRoute.LevelRestrictionType.FlightLevelOdd => actual == PreferredRoute.LevelRestrictionType.FlightLevelOdd,
            PreferredRoute.LevelRestrictionType.FlightLevel => actual == PreferredRoute.LevelRestrictionType.FlightLevel ||
                                                              actual == PreferredRoute.LevelRestrictionType.FlightLevelEven ||
                                                              actual == PreferredRoute.LevelRestrictionType.FlightLevelOdd,
            _ => throw new InvalidOperationException($"Unexpected level restriction type: {expected}"),
        };
    }
}
