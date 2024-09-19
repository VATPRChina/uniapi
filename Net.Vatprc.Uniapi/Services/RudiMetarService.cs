using System.Diagnostics;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class RudiMetarService(IOptions<RudiMetarService.Option> Options)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<RudiMetarService>();
        return builder;
    }

    public async Task<string> GetMetar(string icao)
    {
        var response = await Options.Value.Endpoint
            .SetQueryParam("id", icao)
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetAsync();
        return await response.GetStringAsync();
    }

    public class Option
    {
        public const string LOCATION = "Utils:Metar";

        public required string Endpoint { get; set; }
    }
}
