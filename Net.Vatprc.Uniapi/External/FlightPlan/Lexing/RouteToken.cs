namespace Net.Vatprc.Uniapi.External.FlightPlan.Lexing;

public record class RouteToken
{
    public required RouteTokenKind Kind { get; set; }

    public required string Value { get; set; }

    public required Ulid Id { get; set; }
}
