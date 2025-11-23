using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Adapters;

public class DiscourseAdapter(IOptions<DiscourseAdapter.Option> Options)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<DiscourseAdapter>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Discourse";

        public required string Endpoint { get; set; }
        public required string ApiKey { get; set; }
    }

    public string Endpoint = Options.Value.Endpoint;

    public async Task<CategoryResult> GetNotamTopics()
    {
        return await Options.Value.Endpoint
            .AppendPathSegments("c/69-category/notam/79.json")
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetJsonAsync<CategoryResult>() ??
            throw new Exception("Unexpected null");
    }

    #region "NOTAM Models"
    public class CategoryResult
    {
        [JsonPropertyName("topic_list")]
        public required TopicList TopicList { get; set; }
    }

    public class TopicList
    {
        [JsonPropertyName("topics")]
        public required IEnumerable<Topic> Topics { get; set; }
    }

    public class Topic
    {
        [JsonPropertyName("id")]
        public required uint Id { get; set; }

        [JsonPropertyName("title")]
        public required string Title { get; set; }

        [JsonPropertyName("tags")]
        public IEnumerable<string> Tags { get; set; } = [];
    }
    #endregion

    public async Task<CalendarEvents> GetCalendarEvents()
    {
        return await Options.Value.Endpoint
            .AppendPathSegments("discourse-post-event/events.json")
            .AppendQueryParam("category_id", "66")
            .AppendQueryParam("include_subcategories", "true")
            .AppendQueryParam("include_expired", "true")
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetJsonAsync<CalendarEvents>() ??
            throw new Exception("Unexpected null");
    }

    #region "Calendar Events Models"
    public class CalendarEvents
    {
        [JsonPropertyName("events")]
        public required Event[] Events { get; set; }
    }

    public class Event
    {
        [JsonPropertyName("id")]
        public required long Id { get; set; }

        [JsonPropertyName("starts_at")]
        public required DateTimeOffset StartsAt { get; set; }

        [JsonPropertyName("ends_at")]
        public required DateTimeOffset EndsAt { get; set; }

        [JsonPropertyName("timezone")]
        public required string Timezone { get; set; }

        [JsonPropertyName("post")]
        public required Post Post { get; set; }

        [JsonPropertyName("name")]
        public required string Name { get; set; }

        [JsonPropertyName("category_id")]
        public required long CategoryId { get; set; }
    }

    public class Post
    {
        [JsonPropertyName("id")]
        public required long Id { get; set; }

        [JsonPropertyName("post_number")]
        public required long PostNumber { get; set; }

        [JsonPropertyName("url")]
        public required string Url { get; set; }

        [JsonPropertyName("topic")]
        public required Topic Topic { get; set; }
    }
    #endregion
}
