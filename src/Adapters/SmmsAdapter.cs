using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Adapters;

public class SmmsAdapter(IOptions<SmmsAdapter.Option> Options)
{
    public const string BASE_URL = "https://s.ee/api/v1/file";

    public async Task<string> UploadImageAsync(Stream imageStream, string fileName, CancellationToken ct = default)
    {
        if (fileName.Any(c => !char.IsLetterOrDigit(c)))
        {
            fileName = Ulid.NewUlid().ToString();
        }
        var response = await BASE_URL
            .AppendPathSegment("upload")
            .WithHeader("Authorization", Options.Value.SecretToken)
            .PostMultipartAsync(mp => mp
                .AddFile("file", imageStream, $"vatprc-{DateTimeOffset.UtcNow:yyyy-MM-dd}-{fileName}"),
                cancellationToken: ct)
            .ReceiveJson<SmmsResponse>();
        if (response.Code != 200)
        {
            throw new Exception("Image upload to SM.MS failed: " + response.Message);
        }
        return response.Data?.Url ?? throw new Exception("Image upload to SM.MS failed: No URL returned");
    }

    public class SmmsResponse
    {
        [JsonPropertyName("code")]
        public long Code { get; set; }

        [JsonPropertyName("data")]
        public SmmsData? Data { get; set; }

        [JsonPropertyName("message")]
        public string Message { get; set; } = string.Empty;
    }

    public class SmmsData
    {
        [JsonPropertyName("url")]
        public string Url { get; set; } = string.Empty;
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<SmmsAdapter>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Storage:Image:Smms";

        public required string SecretToken { get; set; }
    }
}
