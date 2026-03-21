using System.Text.Json.Serialization;

namespace Net.Vatprc.Uniapi.Adapters.Moodle;

public record MoodleCustomField
{
    [JsonPropertyName("type")]
    public required string Type { get; init; }

    [JsonPropertyName("value")]
    public required string Value { get; init; }

    [JsonPropertyName("displayvalue")]
    public string? DisplayValue { get; init; }

    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("shortname")]
    public required string ShortName { get; init; }
}

public record MoodlePreference
{
    [JsonPropertyName("name")]
    public required string Name { get; init; }

    [JsonPropertyName("value")]
    public required string Value { get; init; }
}

public record MoodleUser
{
    [JsonPropertyName("id")]
    public int Id { get; init; }

    [JsonPropertyName("username")]
    public string? Username { get; init; }

    [JsonPropertyName("firstname")]
    public string? FirstName { get; init; }

    [JsonPropertyName("lastname")]
    public string? LastName { get; init; }

    [JsonPropertyName("fullname")]
    public required string FullName { get; init; }

    [JsonPropertyName("email")]
    public string? Email { get; init; }

    [JsonPropertyName("address")]
    public string? Address { get; init; }

    [JsonPropertyName("phone1")]
    public string? Phone1 { get; init; }

    [JsonPropertyName("phone2")]
    public string? Phone2 { get; init; }

    [JsonPropertyName("department")]
    public string? Department { get; init; }

    [JsonPropertyName("institution")]
    public string? Institution { get; init; }

    [JsonPropertyName("idnumber")]
    public string? IdNumber { get; init; }

    [JsonPropertyName("interests")]
    public string? Interests { get; init; }

    [JsonPropertyName("firstaccess")]
    public int? FirstAccess { get; init; }

    [JsonPropertyName("lastaccess")]
    public int? LastAccess { get; init; }

    [JsonPropertyName("auth")]
    public string? Auth { get; init; }

    [JsonPropertyName("suspended")]
    public bool? Suspended { get; init; }

    [JsonPropertyName("confirmed")]
    public bool? Confirmed { get; init; }

    [JsonPropertyName("lang")]
    public string? Lang { get; init; }

    [JsonPropertyName("calendartype")]
    public string? CalendarType { get; init; }

    [JsonPropertyName("theme")]
    public string? Theme { get; init; }

    [JsonPropertyName("timezone")]
    public string? Timezone { get; init; }

    [JsonPropertyName("mailformat")]
    public int? MailFormat { get; init; }

    [JsonPropertyName("trackforums")]
    public int? TrackForums { get; init; }

    [JsonPropertyName("description")]
    public string? Description { get; init; }

    [JsonPropertyName("descriptionformat")]
    public int? DescriptionFormat { get; init; }

    [JsonPropertyName("city")]
    public string? City { get; init; }

    [JsonPropertyName("country")]
    public string? Country { get; init; }

    [JsonPropertyName("profileimageurlsmall")]
    public required string ProfileImageUrlSmall { get; init; }

    [JsonPropertyName("profileimageurl")]
    public required string ProfileImageUrl { get; init; }

    [JsonPropertyName("customfields")]
    public List<MoodleCustomField>? CustomFields { get; init; }

    [JsonPropertyName("preferences")]
    public List<MoodlePreference>? Preferences { get; init; }
}

public record MoodleCreateUserResponseItem
{
    [JsonPropertyName("id")]
    public required int Id { get; init; }

    [JsonPropertyName("username")]
    public required string Username { get; init; }
}

