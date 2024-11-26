using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class DiscourseService(IOptions<DiscourseService.Option> Options)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<DiscourseService>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Discourse";

        public required string Endpoint { get; set; }
    }

    public string Endpoint = Options.Value.Endpoint;

    public async Task<CategoryResult> GetNotamTopics()
    {
        return await Options.Value.Endpoint
            .AppendPathSegments("c/69-category/notam/79.json")
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetJsonAsync<CategoryResult>() ??
            throw new Exception("Unexpected null on fetch vatprc atcapi data");
    }


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
        public required IEnumerable<string> Tags { get; set; }
    }
}
