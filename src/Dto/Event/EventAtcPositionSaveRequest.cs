using static Net.Vatprc.Uniapi.Models.Atc.UserAtcPermission;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAtcPositionSaveRequest
{
    public required string Callsign { get; set; }
    public required DateTimeOffset StartAt { get; set; }
    public required DateTimeOffset EndAt { get; set; }
    public string? Remarks { get; set; }
    public required string PositionKindId { get; set; }
    public required UserControllerState MinimumControllerState { get; set; }
}
