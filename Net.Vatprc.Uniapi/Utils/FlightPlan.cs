using System.Text.RegularExpressions;

namespace Net.Vatprc.Uniapi.Utils;

public static partial class FlightPlan
{
    public record Aircraft(
        string AircraftCode,
        string Equipment,
        string Transponder,
        string NavigationPerformance
    );

    [GeneratedRegex(@"^(?<code>[A-Z0-9]+)(\/(?<tail>[A-Z])-(?<equip>[A-Z0-9]+)\/(?<trans>[A-Z0-9]+))?$")]
    private static partial Regex AircraftCodeRegex();

    [GeneratedRegex(@"PBN\/(?<pbn>[A-Z0-9]+)( |$)")]
    private static partial Regex PbnCodeRegex();

    public static Aircraft ParseIcaoAircraftCode(string aircraft, string remarks)
    {
        var aircraftNorm = aircraft.ToUpperInvariant();
        var match = AircraftCodeRegex().Match(aircraftNorm);

        match.Groups.TryGetValue("code", out var aircraftCodeGroup);
        match.Groups.TryGetValue("tail", out var tailCodeGroup);
        match.Groups.TryGetValue("equip", out var equipmentGroup);
        match.Groups.TryGetValue("trans", out var transponderGroup);

        var aircraftCode = aircraftCodeGroup?.Value ?? string.Empty;
        var tailCode = tailCodeGroup?.Value ?? string.Empty;
        var equipment = equipmentGroup?.Value ?? string.Empty;
        var transponder = transponderGroup?.Value ?? string.Empty;

        var pbnMatch = PbnCodeRegex().Match(remarks);
        pbnMatch.Groups.TryGetValue("pbn", out var pbnGroup);
        var pbn = pbnGroup?.Value ?? string.Empty;
        return new Aircraft(aircraftCode, equipment, transponder, pbn);
    }
}
