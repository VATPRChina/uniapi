using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAirspaceDto
{
    public required Ulid Id { get; init; }
    public required Ulid EventId { get; init; }
    public required string Name { get; init; }
    public required DateTimeOffset CreatedAt { get; init; }
    public required DateTimeOffset UpdatedAt { get; init; }
    public required IEnumerable<string> IcaoCodes { get; init; }
    public required string Description { get; init; }

    public static EventAirspaceDto From(EventAirspace airspace)
    {
        return new()
        {
            Id = airspace.Id,
            EventId = airspace.EventId,
            Name = airspace.Name,
            CreatedAt = airspace.CreatedAt,
            UpdatedAt = airspace.UpdatedAt,
            IcaoCodes = airspace.IcaoCodes,
            Description = airspace.Description,
        };
    }
}
