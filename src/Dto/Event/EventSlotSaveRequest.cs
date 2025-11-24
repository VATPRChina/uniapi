using System.ComponentModel;

namespace Net.Vatprc.Uniapi.Dto;

public record EventSlotSaveRequest
{
    [Description("Only applies to create. Ignored on update.")]
    public required Ulid AirspaceId { get; set; }
    public required DateTimeOffset EnterAt { get; set; }
    public DateTimeOffset? LeaveAt { get; set; }
    public string? Callsign { get; set; }
    public string? AircraftTypeIcao { get; set; }
}
