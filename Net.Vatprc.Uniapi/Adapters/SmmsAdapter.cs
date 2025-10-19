using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Adapters;

public class SmmsAdapter(IOptions<SmmsAdapter.Option> Options)
{
    public const string BASE_URL = "https://sm.ms/api/v2/";

    public async Task<string> UploadImageAsync(Stream imageStream, string fileName, CancellationToken ct = default)
    {
        var response = await BASE_URL
            .AppendPathSegment("upload")
            .WithHeader("Authorization", Options.Value.SecretToken)
            .PostMultipartAsync(mp => mp
                .AddFile("smfile", imageStream, $"vatprc-{DateTimeOffset.UtcNow:yyyy-MM-dd}-{fileName}"), cancellationToken: ct)
            .ReceiveJson<SmmsResponse>();
        if (!response.Success)
        {
            throw new Exception("Image upload to SM.MS failed");
        }
        return response.Data.Url;
    }

    public class SmmsResponse
    {
        public required bool Success { get; set; }
        public required SmmsData Data { get; set; }
    }

    public class SmmsData
    {
        public required string Url { get; set; }
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
