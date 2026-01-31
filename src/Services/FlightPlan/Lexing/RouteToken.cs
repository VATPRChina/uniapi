using Arinc424;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Lexing;

public record class RouteToken
{
    public required RouteTokenKind Kind { get; set; }

    public required string Value { get; set; }

    public required string Id { get; set; }

    public required Geo? Geo { get; set; }
}
