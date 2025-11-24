namespace Net.Vatprc.Uniapi.Dto;

public class CompatVatprcStatusDto
{
    public required DateTimeOffset LastUpdated { get; set; }
    public required IEnumerable<CompatPilotDto> Pilots { get; set; }
    public required IEnumerable<CompatControllerDto> Controllers { get; set; }
    public required IEnumerable<CompatFutureControllerDto> FutureControllers { get; set; }
}
