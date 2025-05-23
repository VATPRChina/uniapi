namespace Net.Vatprc.Uniapi.Utils;

public static class Geography
{
    /// <summary>
    /// Parse coordinate from CAAC format.
    /// </summary>
    /// <param name="coordinate">XDDDMMSS</param>
    /// <returns></returns>
    public static double ParseCaacCoordinate(string coordinate)
    {
        if (string.IsNullOrEmpty(coordinate))
        {
            return 0;
        }

        var sign = coordinate.StartsWith('S') || coordinate.StartsWith('W') ? -1 : 1;

        var degrees = double.Parse(coordinate[1..^4]);
        var minutes = double.Parse(coordinate[^4..^2]);
        var seconds = double.Parse(coordinate[^2..]);

        return sign * (degrees + minutes / 60 + seconds / 3600);
    }

    public static int ConvertMeterToFeet(int meter)
    {
        return (int)(meter * 3.28084);
    }

    /// <summary>
    /// Compute distance between two points in WGS-84 in nautical miles.
    /// </summary>
    /// <param name="lat1"></param>
    /// <param name="lon1"></param>
    /// <param name="lat2"></param>
    /// <param name="lon2"></param>
    /// <returns></returns>
    public static double DistanceBetweenPoints(double lat1, double lon1, double lat2, double lon2)
    {
        var R = 6371; // Radius of the earth in km
        var dLat = double.DegreesToRadians(lat2 - lat1);
        var dLon = double.DegreesToRadians(lon2 - lon1);
        var a = Math.Sin(dLat / 2) * Math.Sin(dLat / 2) +
                Math.Cos(double.DegreesToRadians(lat1)) * Math.Cos(double.DegreesToRadians(lat2)) *
                Math.Sin(dLon / 2) * Math.Sin(dLon / 2);
        var c = 2 * Math.Atan2(Math.Sqrt(a), Math.Sqrt(1 - a));
        var d = R * c; // Distance in km
        return d * 0.539957; // Distance in nautical miles
    }

    public static double NauticalMileToMeter(double nm)
    {
        return nm * 1852;
    }

    /// <summary>
    /// Parse coordinate from ARINC format. (Section 5.36, 5.37)
    ///
    /// For latitude, `XDDMMSSss`; for longitude, `XDDDMMSSss`.
    /// </summary>
    /// <param name="coordinate"></param>
    /// <returns></returns>
    public static double ParseArincCoordinate(string coordinate)
    {
        if (string.IsNullOrEmpty(coordinate))
        {
            return 0;
        }

        if (coordinate.StartsWith('N') || coordinate.StartsWith('S'))
        {
            var sign = coordinate.StartsWith('S') ? -1 : 1;
            var degrees = (double)int.Parse(coordinate[1..3]);
            var minutes = (double)int.Parse(coordinate[3..5]);
            var secondHundredths = (double)int.Parse(coordinate[5..]);
            return sign * (degrees + minutes / 60 + secondHundredths / (3600 * 100));
        }

        if (coordinate.StartsWith('E') || coordinate.StartsWith('W'))
        {
            var sign = coordinate.StartsWith('W') ? -1 : 1;
            var degrees = (double)int.Parse(coordinate[1..4]);
            var minutes = (double)int.Parse(coordinate[4..6]);
            var secondHundredths = (double)int.Parse(coordinate[6..]);
            return sign * (degrees + minutes / 60 + secondHundredths / (3600 * 100));
        }

        throw new ArgumentException($"Invalid coordinate format: {coordinate}");
    }
}
