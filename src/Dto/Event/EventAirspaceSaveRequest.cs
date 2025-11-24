namespace Net.Vatprc.Uniapi.Dto;

public record EventAirspaceSaveRequest
{
    public required string Name { get; set; }
    public required IEnumerable<string> IcaoCodes { get; set; }
    public required string Description { get; set; }
}
