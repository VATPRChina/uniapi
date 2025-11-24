namespace Net.Vatprc.Uniapi.Dto;

public class CompatPilotDto
{
    public required int Cid { get; set; }
    public required string Name { get; set; }
    public required string Callsign { get; set; }
    public required string? Departure { get; set; }
    public required string? Arrival { get; set; }
    public required string? Aircraft { get; set; }
}
