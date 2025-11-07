using System.Diagnostics.CodeAnalysis;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Utility;

/// <summary>
/// Cruise altitude parser for MH/T 4007—2023 5.13 Cruise Altitude
/// </summary>
public class CruiseAltitudeParser
{
    public const int MaxTokenLength = 5;
    public const int MinTokenLength = 4;

    /// <summary>
    /// Kind of cruise altitude parsed from the flight plan.
    /// </summary>
    public enum Kind
    {
        /// <summary>Altitude in 10 metres units (Mnnnn). Value (meters) = digits * 10</summary>
        MetricAltitude,
        /// <summary>Flight level represented in 10 metres units (Snnnn). Value (meters) = digits * 10</summary>
        MetricFlightLevel,
        /// <summary>Altitude in 100 feet units (Annn). Value (feet) = digits * 100</summary>
        ImperialAltitude,
        /// <summary>Flight level represented in 100 feet units (Fnnn). Value (feet) = digits * 100</summary>
        ImperialFlightLevel,
    }

    /// <summary>
    /// Strongly-typed result of a cruise altitude parse.
    /// </summary>
    /// <param name="Kind">The kind of the parsed altitude token.</param>
    /// <param name="Value">The numeric value as an integer. Interpretation depends on <paramref name="Kind"/> (see docs on <see cref="Kind"/>).</param>
    /// <param name="Raw">Original raw token parsed.</param>
    public sealed record CruiseAltitudeResult(Kind Kind, int Value, string Raw)
    {
        public override string ToString() => Raw;
    }

    /// <summary>
    /// Try to parse a cruise altitude token.
    /// </summary>
    /// <param name="token">Token to parse (case-insensitive).</param>
    /// <param name="result">Parsed result when method returns true.</param>
    /// <returns>True when parsing succeeded; otherwise false.</returns>
    public static bool TryParse(string? token, [NotNullWhen(true)] out CruiseAltitudeResult? result)
    {
        result = null;
        if (string.IsNullOrWhiteSpace(token)) return false;

        var t = token.Trim().ToUpperInvariant();

        // Mnnnn or Snnnn -> 4 digits after letter, value in 10 meters
        if (t.Length == 5 && (t[0] == 'M' || t[0] == 'S'))
        {
            var digits = t[1..];
            if (digits.All(char.IsDigit))
            {
                if (!int.TryParse(digits, out var n)) return false;
                var valueMeters = n * 10; // meters
                var kind = t[0] == 'M' ? Kind.MetricAltitude : Kind.MetricFlightLevel;
                result = new CruiseAltitudeResult(kind, valueMeters, t);
                return true;
            }
            return false;
        }

        // Annn or Fnnn -> 3 digits after letter, value in 100 feet
        if (t.Length == 4 && (t[0] == 'A' || t[0] == 'F'))
        {
            var digits = t[1..];
            if (digits.All(char.IsDigit))
            {
                if (!int.TryParse(digits, out var n)) return false;
                var valueFeet = n * 100; // feet
                var kind = t[0] == 'A' ? Kind.ImperialAltitude : Kind.ImperialFlightLevel;
                result = new CruiseAltitudeResult(kind, valueFeet, t);
                return true;
            }
            return false;
        }

        return false;
    }
}
