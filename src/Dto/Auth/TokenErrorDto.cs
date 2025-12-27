namespace Net.Vatprc.Uniapi.Dto;

public record TokenErrorDto
{
    public required string Error { get; set; }
    public string? ErrorDescription { get; set; }
    public string? ErrorUri { get; set; }
    public string? State { get; set; }
}
