#pragma warning disable CS8618

using System.Text.Json;
using System.Text.Json.Serialization;
using System.Globalization;

namespace Net.Vatprc.Uniapi.Services.VatsimData;

public partial class VatsimData
{
    [JsonPropertyName("general")]
    public General General { get; set; }

    [JsonPropertyName("pilots")]
    public Pilot[] Pilots { get; set; }

    [JsonPropertyName("controllers")]
    public Atc[] Controllers { get; set; }

    [JsonPropertyName("atis")]
    public Atc[] Atis { get; set; }

    [JsonPropertyName("servers")]
    public ServerElement[] Servers { get; set; }

    [JsonPropertyName("prefiles")]
    public Prefile[] Prefiles { get; set; }

    [JsonPropertyName("facilities")]
    public Facility[] Facilities { get; set; }

    [JsonPropertyName("ratings")]
    public Facility[] Ratings { get; set; }

    [JsonPropertyName("pilot_ratings")]
    public Rating[] PilotRatings { get; set; }

    [JsonPropertyName("military_ratings")]
    public Rating[] MilitaryRatings { get; set; }
}

public partial class Atc
{
    [JsonPropertyName("cid")]
    public long Cid { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; }

    [JsonPropertyName("callsign")]
    public string Callsign { get; set; }

    [JsonPropertyName("frequency")]
    public string Frequency { get; set; }

    [JsonPropertyName("facility")]
    public long Facility { get; set; }

    [JsonPropertyName("rating")]
    public long Rating { get; set; }

    [JsonPropertyName("server")]
    public string Server { get; set; }

    [JsonPropertyName("visual_range")]
    public long VisualRange { get; set; }

    [JsonPropertyName("atis_code")]
    public string AtisCode { get; set; }

    [JsonPropertyName("text_atis")]
    public string[] TextAtis { get; set; }

    [JsonPropertyName("last_updated")]
    public DateTimeOffset LastUpdated { get; set; }

    [JsonPropertyName("logon_time")]
    public DateTimeOffset LogonTime { get; set; }
}

public partial class Facility
{
    [JsonPropertyName("id")]
    public long Id { get; set; }

    [JsonPropertyName("short")]
    public string Short { get; set; }

    [JsonPropertyName("long")]
    public string Long { get; set; }
}

public partial class General
{
    [JsonPropertyName("version")]
    public long Version { get; set; }

    [JsonPropertyName("reload")]
    public long Reload { get; set; }

    [JsonPropertyName("update")]
    public string Update { get; set; }

    [JsonPropertyName("update_timestamp")]
    public DateTimeOffset UpdateTimestamp { get; set; }

    [JsonPropertyName("connected_clients")]
    public long ConnectedClients { get; set; }

    [JsonPropertyName("unique_users")]
    public long UniqueUsers { get; set; }
}

public partial class Rating
{
    [JsonPropertyName("id")]
    public long Id { get; set; }

    [JsonPropertyName("short_name")]
    public string ShortName { get; set; }

    [JsonPropertyName("long_name")]
    public string LongName { get; set; }
}

public partial class Pilot
{
    [JsonPropertyName("cid")]
    public long Cid { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; }

    [JsonPropertyName("callsign")]
    public string Callsign { get; set; }

    [JsonPropertyName("server")]
    public string Server { get; set; }

    [JsonPropertyName("pilot_rating")]
    public long PilotRating { get; set; }

    [JsonPropertyName("military_rating")]
    public long MilitaryRating { get; set; }

    [JsonPropertyName("latitude")]
    public double Latitude { get; set; }

    [JsonPropertyName("longitude")]
    public double Longitude { get; set; }

    [JsonPropertyName("altitude")]
    public long Altitude { get; set; }

    [JsonPropertyName("groundspeed")]
    public long Groundspeed { get; set; }

    [JsonPropertyName("transponder")]
    public string Transponder { get; set; }

    [JsonPropertyName("heading")]
    public long Heading { get; set; }

    [JsonPropertyName("qnh_i_hg")]
    public double QnhIHg { get; set; }

    [JsonPropertyName("qnh_mb")]
    public long QnhMb { get; set; }

    [JsonPropertyName("flight_plan")]
    public FlightPlan? FlightPlan { get; set; }

    [JsonPropertyName("logon_time")]
    public DateTimeOffset LogonTime { get; set; }

    [JsonPropertyName("last_updated")]
    public DateTimeOffset LastUpdated { get; set; }
}

public partial class FlightPlan
{
    [JsonPropertyName("flight_rules")]
    public string FlightRules { get; set; }

    [JsonPropertyName("aircraft")]
    public string Aircraft { get; set; }

    [JsonPropertyName("aircraft_faa")]
    public string AircraftFaa { get; set; }

    [JsonPropertyName("aircraft_short")]
    public string AircraftShort { get; set; }

    [JsonPropertyName("departure")]
    public string Departure { get; set; }

    [JsonPropertyName("arrival")]
    public string Arrival { get; set; }

    [JsonPropertyName("alternate")]
    public string Alternate { get; set; }

    [JsonPropertyName("cruise_tas")]
    public string CruiseTas { get; set; }

    [JsonPropertyName("altitude")]
    public string Altitude { get; set; }

    [JsonPropertyName("deptime")]
    public string Deptime { get; set; }

    [JsonPropertyName("enroute_time")]
    public string EnrouteTime { get; set; }

    [JsonPropertyName("fuel_time")]
    public string FuelTime { get; set; }

    [JsonPropertyName("remarks")]
    public string Remarks { get; set; }

    [JsonPropertyName("route")]
    public string Route { get; set; }

    [JsonPropertyName("revision_id")]
    public long RevisionId { get; set; }

    [JsonPropertyName("assigned_transponder")]
    public string AssignedTransponder { get; set; }
}

public partial class Prefile
{
    [JsonPropertyName("cid")]
    public long Cid { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; }

    [JsonPropertyName("callsign")]
    public string Callsign { get; set; }

    [JsonPropertyName("flight_plan")]
    public FlightPlan FlightPlan { get; set; }

    [JsonPropertyName("last_updated")]
    public DateTimeOffset LastUpdated { get; set; }
}

public partial class ServerElement
{
    [JsonPropertyName("ident")]
    public string Ident { get; set; }

    [JsonPropertyName("hostname_or_ip")]
    public string HostnameOrIp { get; set; }

    [JsonPropertyName("location")]
    public string Location { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; }

    [JsonPropertyName("clients_connection_allowed")]
    public long ClientsConnectionAllowed { get; set; }

    [JsonPropertyName("client_connections_allowed")]
    public bool ClientConnectionsAllowed { get; set; }

    [JsonPropertyName("is_sweatbox")]
    public bool IsSweatbox { get; set; }
}

public partial class AtcSchedule
{
    [JsonPropertyName("callsign")]
    public string Callsign { get; set; }

    [JsonPropertyName("start")]
    public DateTimeOffset Start { get; set; }

    [JsonPropertyName("finish")]
    public DateTimeOffset Finish { get; set; }

    [JsonPropertyName("remark")]
    public string Remark { get; set; }

    [JsonPropertyName("created_at")]
    public DateTimeOffset CreatedAt { get; set; }

    [JsonPropertyName("updated_at")]
    public DateTimeOffset UpdatedAt { get; set; }

    [JsonPropertyName("user")]
    public User User { get; set; }
}

public partial class User
{
    [JsonPropertyName("id")]
    public long Id { get; set; }

    [JsonPropertyName("first_name")]
    public string FirstName { get; set; }

    [JsonPropertyName("last_name")]
    public string LastName { get; set; }
}
