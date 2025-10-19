namespace Net.Vatprc.Uniapi.Utils;

public static class FlightPlanUtils
{
    public record Aircraft(
        string AircraftCode,
        string Equipment,
        string Transponder,
        string NavigationPerformance
    );

    public static string GetPbn(string remarks)
    {
        var iPbn = remarks.IndexOf("PBN/");
        if (iPbn < 0) return string.Empty;
        iPbn += "PBN/".Length;

        var iEnd = remarks.IndexOf(' ', iPbn);
        if (iEnd == -1) iEnd = remarks.Length - 1;

        return remarks[iPbn..iEnd];
    }

    public static Aircraft ParseIcaoAircraftCode(string aircraft, string remarks)
    {
        var aircraftNorm = aircraft.ToUpperInvariant();
        var segments = aircraftNorm.Split('/');

        var code = segments[0];
        var tail = segments.Length > 1 ? segments[1].Split('-')[0] : string.Empty;
        var equipment = segments.Length > 1 ? segments[1].Split('-')[1] : string.Empty;
        var transponder = segments.Length > 2 ? segments[2] : string.Empty;

        var pbn = GetPbn(remarks);

        return new Aircraft(code, equipment, transponder, pbn);
    }
}
