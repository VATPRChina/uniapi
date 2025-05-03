using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class RudiMetarService(IOptions<RudiMetarService.Option> Options)
{
    protected MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());
    protected string LastData { get; set; } = string.Empty;

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<RudiMetarService>();
        return builder;
    }

    public async Task<string> GetMetarDatabaseAsync(CancellationToken ct = default)
    {
        try
        {
            var data = await Cache.GetOrCreateAsync("rudi-metar", async entry =>
            {
                entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
                return await Options.Value.Endpoint
                        .WithHeader("User-Agent", UniapiUserAgent)
                        .WithTimeout(15)
                        .GetStringAsync(cancellationToken: ct);
            });
            LastData = data ?? string.Empty;
            return LastData;
        }
        catch (FlurlHttpException e)
        {
            e.SetSentryMechanism(nameof(RudiMetarService), handled: false);
            SentrySdk.CaptureException(e);
            return LastData;
        }
    }

    public async Task<string> GetMetar(string icao, CancellationToken ct = default)
    {
        var result = await GetMetarDatabaseAsync(ct);
        var metars = result
            .Split('\n')
            .Where(x => x.StartsWith(icao, StringComparison.OrdinalIgnoreCase));
        var metar = string.Join('\n', metars);
        if (string.IsNullOrEmpty(metar))
        {
            metar = await "https://metar.vatsim.net/"
                .WithHeader("User-Agent", UniapiUserAgent)
                .AppendPathSegment(icao)
                .WithTimeout(15)
                .GetStringAsync(cancellationToken: ct);
        }
        return metar;
    }

    public class Option
    {
        public const string LOCATION = "Utils:Metar";

        public required string Endpoint { get; set; }
    }
}
