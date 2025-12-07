using Net.Vatprc.Uniapi.Models.Navdata;
using static Net.Vatprc.Uniapi.Models.Navdata.PreferredRoute;

namespace Net.Vatprc.Uniapi.Dto.Navdata;

public record PreferredRouteDto
{
    public required Ulid Id { get; set; }
    public required string Departure { get; set; }
    public required string Arrival { get; set; }
    public required string RawRoute { get; set; }
    public required LevelRestrictionType CruisingLevelRestriction { get; set; }
    public required IEnumerable<int> AllowedAltitudes { get; set; }
    public required int MinimalAltitude { get; set; }
    public required string Remarks { get; set; }
    public required DateTimeOffset? ValidFrom { get; set; }
    public required DateTimeOffset? ValidUntil { get; set; }

    public static PreferredRouteDto FromModel(PreferredRoute route) => new()
    {
        Id = route.Id,
        Departure = route.Departure,
        Arrival = route.Arrival,
        RawRoute = route.RawRoute,
        CruisingLevelRestriction = route.CruisingLevelRestriction,
        AllowedAltitudes = route.AllowedAltitudes,
        MinimalAltitude = route.MinimalAltitude,
        Remarks = route.Remarks,
        ValidFrom = route.ValidFrom,
        ValidUntil = route.ValidUntil,
    };
}
