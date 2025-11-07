using System.Diagnostics.CodeAnalysis;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

/// <summary>
/// Parser for MH/T 4007—2023 5.12 Cruise Speed
/// </summary>
public class CruiseSpeedParser
{
    public const int MaxTokenLength = 5;
    public const int MinTokenLength = 3;

    /// <summary>
    /// Kind of cruise speed parsed from the flight plan.
    /// </summary>
    public enum Kind
    {
        /// <summary>True airspeed in kilometres per hour (Knnnn)</summary>
        Kph,
        /// <summary>True airspeed in nautical miles per hour / knots (Nnnnn)</summary>
        Kts,
        /// <summary>Mach number with 1% precision (Mnnn where value = nnn / 100)</summary>
        Mach,
        /// <summary>Visual Flight Rules (VFR) special value.</summary>
        VFR,
    }

    /// <summary>
    /// Strongly-typed result of a cruise speed parse.
    /// </summary>
    /// <param name="Kind">The kind of the parsed speed.</param>
    /// <param name="Value">The numeric value. For <see cref="Kind.Mach"/> this is the mach value (e.g. 0.80). For Kph/Kts this is the integer speed.</param>
    /// <param name="Raw">Original raw token parsed.</param>
    public sealed record CruiseSpeedResult(Kind Kind, double Value, string Raw)
    {
        public override string ToString() => Raw;
    }

    /// <summary>
    /// Try to parse a cruise speed token.
    /// Supported formats:
    /// - K followed by 4 digits (Knnnn) => kilometres per hour
    /// - N followed by 4 digits (Nnnnn) => nautical miles per hour (knots)
    /// - M followed by 3 digits (Mnnn) => mach number in 1% precision (nnn / 100)
    /// - VFR => Visual Flight Rules special value
    /// </summary>
    /// <param name="token">Token to parse (case-insensitive).</param>
    /// <param name="result">Parsed result when method returns true.</param>
    /// <returns>True when parsing succeeded; otherwise false.</returns>
    public static bool TryParse(string? token, [NotNullWhen(true)] out CruiseSpeedResult? result)
    {
        result = null;
        if (string.IsNullOrWhiteSpace(token)) return false;

        var t = token.Trim().ToUpperInvariant();

        if (t.Length >= 5 && (t[0] == 'K' || t[0] == 'N'))
        {
            // Knnnn or Nnnnn -> 4 digits after letter
            var digits = t[1..5];
            if (digits.All(char.IsDigit))
            {
                if (!int.TryParse(digits, out var v)) return false;
                var kind = t[0] == 'K' ? Kind.Kph : Kind.Kts;
                result = new CruiseSpeedResult(kind, v, t[0..5]);
                return true;
            }
            return false;
        }

        if (t.Length >= 4 && t[0] == 'M')
        {
            // Mnnn -> 3 digits, mach = nnn / 100
            var digits = t[1..4];
            if (digits.All(char.IsDigit))
            {
                if (!int.TryParse(digits, out var n)) return false;
                var mach = n / 100.0;
                result = new CruiseSpeedResult(Kind.Mach, mach, t[0..4]);
                return true;
            }
            return false;
        }

        if (t.StartsWith("VFR"))
        {
            result = new CruiseSpeedResult(Kind.VFR, 0, t[0..3]);
            return true;
        }

        return false;
    }
}
