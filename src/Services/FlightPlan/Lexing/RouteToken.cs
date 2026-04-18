using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

public abstract record class RouteToken
{
    public required string Value { get; set; }

    public string Kind => GetType().Name;
}

public record class UnknownToken : RouteToken
{
}

public record class FixToken : RouteToken
{
    public required Fix Fix { get; set; }
}
public record class LegToken : RouteToken
{
}

public record class DirectLegToken : LegToken
{
}

public record class AirwayLegToken : LegToken
{
}

public record class SidLegToken : LegToken
{
    public required Procedure? Procedure { get; set; }
}

public record class StarLegToken : LegToken
{
    public required Procedure? Procedure { get; set; }
}

public record class SpeedAndAltitudeToken : RouteToken
{
}
