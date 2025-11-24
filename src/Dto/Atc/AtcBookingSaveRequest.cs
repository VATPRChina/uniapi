namespace Net.Vatprc.Uniapi.Dto;

public class AtcBookingSaveRequest
{
    public required string Callsign { get; init; }
    public required DateTimeOffset StartTime { get; init; }
    public required DateTimeOffset EndTime { get; init; }
}
