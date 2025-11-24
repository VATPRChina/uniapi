using Net.Vatprc.Uniapi.Models.Event;

namespace Net.Vatprc.Uniapi.Dto;

public record EventAirspaceDto(
    Ulid Id,
    Ulid EventId,
    string Name,
    DateTimeOffset CreatedAt,
    DateTimeOffset UpdatedAt,
    IEnumerable<string> IcaoCodes,
    string Description)
{
    public EventAirspaceDto(EventAirspace airspace) : this(
        airspace.Id,
        airspace.EventId,
        airspace.Name,
        airspace.CreatedAt,
        airspace.UpdatedAt,
        airspace.IcaoCodes,
        airspace.Description)
    {
    }
}
