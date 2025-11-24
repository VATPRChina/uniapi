namespace Net.Vatprc.Uniapi.Dto;

public record TokenDto(
    UserDto User,
    DateTimeOffset IssuedAt,
    DateTimeOffset ExpiresAt
);
