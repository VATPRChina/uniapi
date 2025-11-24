namespace Net.Vatprc.Uniapi.Dto;

public class CompatFutureControllerDto
{
    public required string Callsign { get; set; }
    public required string Name { get; set; }
    public required string Start { get; set; }
    public required DateTimeOffset StartUtc { get; set; }
    public required string End { get; set; }
    public required DateTimeOffset EndUtc { get; set; }
}
