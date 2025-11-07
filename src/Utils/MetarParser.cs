namespace Net.Vatprc.Uniapi.Utils;

public static class MetarParser
{
    public static DateTimeOffset GetMetarTime(string metar)
    {
        if (string.IsNullOrWhiteSpace(metar))
        {
            throw new ArgumentException("METAR string cannot be null or empty.", nameof(metar));
        }

        // METAR time is specified in the second space-separated field
        // and is in the format DDHHMMZ.
        var parts = metar.Split(' ', StringSplitOptions.RemoveEmptyEntries);
        if (parts.Length < 2)
        {
            throw new FormatException("METAR string is not in the expected format.");
        }

        var timePart = parts[1];
        if (!(timePart.Length >= 7 && timePart.EndsWith('Z')))
        {
            throw new FormatException("METAR time is not in the expected format (DDHHMMZ).");
        }

        // Extract day, hour, and minute
        if (!int.TryParse(timePart.AsSpan(0, 2), out var day))
        {
            throw new FormatException("Invalid day in METAR time.");
        }
        if (!int.TryParse(timePart.AsSpan(2, 2), out var hour))
        {
            throw new FormatException("Invalid hour in METAR time.");
        }
        if (!int.TryParse(timePart.AsSpan(4, 2), out var minute))
        {
            throw new FormatException("Invalid minute in METAR time.");
        }

        var refTime = DateTimeOffset.UtcNow;

        if (day > refTime.Day)
        {
            refTime = refTime.AddMonths(-1);
        }

        return new DateTimeOffset(refTime.Year, refTime.Month, day, hour, minute, 0, TimeSpan.Zero);
    }

    public static DateTimeOffset? TryGetMetarTime(string metar)
    {
        try
        {
            return GetMetarTime(metar);
        }
        catch (Exception)
        {
            return null;
        }
    }
}
