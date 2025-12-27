namespace Net.Vatprc.Uniapi.Dto;

public record TokenDto
{
    public required UserDto User { get; init; }
    public required DateTimeOffset IssuedAt { get; init; }
    public required DateTimeOffset ExpiresAt { get; init; }
}
