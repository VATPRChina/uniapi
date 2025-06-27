using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.External;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services;

public class RudiMetarService(
    IOptions<RudiMetarService.Option> Options,
    ILogger<RudiMetarService> Logger)
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
                Logger.LogInformation("Fetching METAR data since cache expired.");
                entry.AbsoluteExpiration = DateTimeOffset.UtcNow.RoundUp(TimeSpan.FromMinutes(15));
                Logger.LogInformation("Set expiration to {Expiration}", entry.AbsoluteExpiration);
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
            Logger.LogWarning(e, "Failed to fetch METAR data. Fallback to last known data.");
            return LastData;
        }
    }

    public async Task<string> GetMetar(string icao, CancellationToken ct = default)
    {
        var result = await GetMetarDatabaseAsync(ct);
        var rudiMetar = result
            .Split('\n')
            .Where(x => x.StartsWith(icao, StringComparison.OrdinalIgnoreCase))
            .FirstOrDefault() ?? string.Empty;
        var vatsimMetar = await "https://metar.vatsim.net/"
            .WithHeader("User-Agent", UniapiUserAgent)
            .AppendPathSegment(icao)
            .WithTimeout(15)
            .GetStringAsync(cancellationToken: ct);
        var rudiMetarTime = MetarParser.TryGetMetarTime(rudiMetar);
        var vatprcMetarTime = MetarParser.TryGetMetarTime(vatsimMetar);
        if (rudiMetarTime == null)
        {
            Logger.LogWarning("Failed to parse METAR time from Rudi's METAR data for {Icao}. Using VATSIM METAR instead.", icao);
            return vatsimMetar;
        }
        else if (vatprcMetarTime == null)
        {
            Logger.LogWarning("Failed to parse METAR time from VATSIM METAR data for {Icao}. Using Rudi's METAR instead.", icao);
            return rudiMetar;
        }
        else if (rudiMetarTime > vatprcMetarTime)
        {
            Logger.LogInformation("Using Rudi's METAR for {Icao} as it is more recent than VATSIM's.", icao);
            return rudiMetar;
        }
        else
        {
            Logger.LogInformation("Using VATSIM's METAR for {Icao} as it is more recent than Rudi's.", icao);
            return vatsimMetar;
        }
    }

    public class Option
    {
        public const string LOCATION = "Utils:Metar";

        public required string Endpoint { get; set; }
    }
}
