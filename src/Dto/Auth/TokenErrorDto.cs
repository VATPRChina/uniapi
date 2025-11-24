namespace Net.Vatprc.Uniapi.Dto;

public record TokenErrorDto
{
    public string Error { get; set; } = string.Empty;
    public string? ErrorDescription { get; set; }
    public string? ErrorUri { get; set; }
    public string? State { get; set; }
}
