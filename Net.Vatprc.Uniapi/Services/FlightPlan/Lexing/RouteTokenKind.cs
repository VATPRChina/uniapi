namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

public enum RouteTokenKind
{
    SID,
    STAR,
    AIRWAY,
    AIRPORT,
    VHF,
    NDB,
    WAYPOINT,
    GEO_COORD,
    UNKNOWN,
}

public static class RouteTokenKindExtensions
{
    public static bool IsFix(this RouteTokenKind kind)
    {
        return kind == RouteTokenKind.WAYPOINT
            || kind == RouteTokenKind.VHF
            || kind == RouteTokenKind.NDB
            || kind == RouteTokenKind.GEO_COORD
            || kind == RouteTokenKind.AIRPORT;
    }

    public static bool IsLeg(this RouteTokenKind kind)
    {
        return kind == RouteTokenKind.SID
            || kind == RouteTokenKind.STAR
            || kind == RouteTokenKind.AIRWAY;
    }
}
