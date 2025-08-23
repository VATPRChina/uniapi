using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.External;

namespace Net.Vatprc.Uniapi.Adapters;

public class MetarAdapter(
    IOptions<MetarAdapter.Option> Options,
    ILogger<MetarAdapter> Logger)
{
    protected MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());
    protected string LastData { get; set; } = string.Empty;

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<MetarAdapter>();
        return builder;
    }

    protected async Task<string> GetRudiMetarAsync(string icao, CancellationToken ct = default)
    {
        try
        {
            var metar = await Options.Value.Endpoint
                .WithHeader("User-Agent", UniapiUserAgent)
                .AppendPathSegment(icao)
                .WithTimeout(15)
                .GetStringAsync(cancellationToken: ct);
            return metar.Trim();

        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Failed to fetch METAR for {Icao} from Rudi's database.", icao);
            SentrySdk.CaptureException(ex, scope =>
            {
                scope.SetTag("Icao", icao);
                scope.SetExtra("Endpoint", Options.Value.Endpoint);
            });
            return string.Empty;
        }
    }

    protected async Task<string> GetVatsimMetarAsync(string icao, CancellationToken ct = default)
    {
        try
        {
            var metar = await "https://metar.vatsim.net/"
                .WithHeader("User-Agent", UniapiUserAgent)
                .AppendPathSegment(icao)
                .WithTimeout(15)
                .GetStringAsync(cancellationToken: ct);
            return metar.Trim();
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Failed to fetch METAR for {Icao} from VATSIM.", icao);
            SentrySdk.CaptureException(ex, scope =>
            {
                scope.SetTag("Icao", icao);
            });
            return string.Empty;
        }
    }

    public async Task<string> GetMetar(string icao, CancellationToken ct = default)
    {
        var rudiMetar = await GetRudiMetarAsync(icao, ct);
        Logger.LogInformation("Fetched METAR for {Icao} from Rudi's database: {Metar}", icao, rudiMetar);
        var vatsimMetar = await GetVatsimMetarAsync(icao, ct);
        Logger.LogInformation("Fetched METAR for {Icao} from VATSIM: {Metar}", icao, vatsimMetar);
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
