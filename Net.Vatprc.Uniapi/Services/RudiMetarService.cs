using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class RudiMetarService(IOptions<RudiMetarService.Option> Options)
{
    MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<RudiMetarService>();
        return builder;
    }

    protected async Task<string> GetMetarDatabaseAsync(CancellationToken ct = default)
    {
        return await Cache.GetOrCreateAsync("rudi-metar", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            try
            {
                var result = await Options.Value.Endpoint
                    .WithHeader("User-Agent", UniapiUserAgent)
                    .GetStringAsync(cancellationToken: ct);
                return result;
            }
            catch (FlurlHttpException e)
            {
                e.SetSentryMechanism(nameof(RudiMetarService), handled: false);
                SentrySdk.CaptureException(e);
                return entry.Value as string ?? string.Empty;
            }
        }) ?? string.Empty;
    }

    public async Task<string> GetMetar(string icao, CancellationToken ct = default)
    {
        var result = await GetMetarDatabaseAsync(ct);
        var metars = result
            .Split('\n')
            .Where(x => x.StartsWith(icao, StringComparison.OrdinalIgnoreCase));
        return string.Join('\n', metars);
    }

    public class Option
    {
        public const string LOCATION = "Utils:Metar";

        public required string Endpoint { get; set; }
    }
}
