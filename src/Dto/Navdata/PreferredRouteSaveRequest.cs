using Net.Vatprc.Uniapi.Models.Navdata;
using static Net.Vatprc.Uniapi.Models.Navdata.PreferredRoute;

namespace Net.Vatprc.Uniapi.Dto.Navdata;

public record PreferredRouteSaveRequest
{
    public required string Departure { get; set; }
    public required string Arrival { get; set; }
    public required string RawRoute { get; set; }
    public required LevelRestrictionType CruisingLevelRestriction { get; set; }
    public IEnumerable<int> AllowedAltitudes { get; set; } = [];
    public required int MinimalAltitude { get; set; }
    public required string Remarks { get; set; }
    public DateTimeOffset? ValidFrom { get; set; }
    public DateTimeOffset? ValidUntil { get; set; }
}
