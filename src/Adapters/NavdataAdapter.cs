using Amazon.Runtime;
using Amazon.S3;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;
using nietras.SeparatedValues;

namespace Net.Vatprc.Uniapi.Adapters;

public class NavadataAdapter(IMemoryCache cache, IOptions<NavadataAdapter.Option> options)
{
    public async Task<string> GetArincData(CancellationToken ct = default)
    {
        return await GetS3FileAsync(options.Value.ArincPath, ct);
    }

    public async Task<string> GetRoutesAsync(CancellationToken ct = default)
    {
        return await GetS3FileAsync(options.Value.RoutePath, ct);
    }

    protected async Task<string> GetS3FileAsync(string path, CancellationToken ct = default)
    {
        var credentials = new BasicAWSCredentials(options.Value.AccessKey, options.Value.SecretKey);
        var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
        {
            ServiceURL = options.Value.ServiceUrl,
        });
        var response = await s3Client.GetObjectAsync(options.Value.Bucket, path, ct);
        using var streamReader = new StreamReader(response.ResponseStream);
        return await streamReader.ReadToEndAsync(ct);
    }

    public async Task<IDictionary<string, IDictionary<string, IList<PreferredRoute>>>> GetPreferredRoutesAsync(CancellationToken ct = default)
    {
        var routesText = await GetRoutesAsync(ct);
        using var reader = Sep.Reader().FromText(routesText);

        var routes = new Dictionary<string, IDictionary<string, IList<PreferredRoute>>>();
        foreach (var row in reader)
        {
            var departure = row["Dep"].Span.ToString();
            var arrival = row["Arr"].Span.ToString();
            var preferredRoute = new PreferredRoute
            {
                Id = Ulid.NewUlid(),
                Departure = departure,
                Arrival = arrival,
                RawRoute = row["Route"].Span.ToString(),
                CruisingLevelRestriction = ParseLevelRestriction(row["EvenOdd"].Span),
                AllowedAltitudes = ParseAllowedAltitudes(row["AltList"].Span),
                MinimalAltitude = ParseMinimalAltitude(row["MinAlt"].Span),
                Remarks = row["Remarks"].Span.ToString(),
            };

            if (!routes.TryGetValue(departure, out var arrivalRoutes))
            {
                arrivalRoutes = new Dictionary<string, IList<PreferredRoute>>();
                routes.Add(departure, arrivalRoutes);
            }

            if (!arrivalRoutes.TryGetValue(arrival, out var preferredRoutes))
            {
                preferredRoutes = [];
                arrivalRoutes.Add(arrival, preferredRoutes);
            }
            preferredRoutes.Add(preferredRoute);
        }

        return routes;
    }

    private static PreferredRoute.LevelRestrictionType ParseLevelRestriction(ReadOnlySpan<char> value)
    {
        return value switch
        {
            "SE" => PreferredRoute.LevelRestrictionType.StandardEven,
            "SO" => PreferredRoute.LevelRestrictionType.StandardOdd,
            "FE" => PreferredRoute.LevelRestrictionType.FlightLevelEven,
            "FO" => PreferredRoute.LevelRestrictionType.FlightLevelOdd,
            _ => PreferredRoute.LevelRestrictionType.Standard,
        };
    }

    private static IList<int> ParseAllowedAltitudes(ReadOnlySpan<char> value)
    {
        var altitudes = new List<int>();
        while (!value.IsEmpty)
        {
            var separatorIndex = value.IndexOf('/');
            var token = separatorIndex >= 0 ? value[..separatorIndex] : value;
            value = separatorIndex >= 0 ? value[(separatorIndex + 1)..] : [];

            if (token.IsWhiteSpace())
            {
                continue;
            }

            altitudes.Add(ParseAllowedAltitude(token));
        }
        return altitudes;
    }

    private static int ParseAllowedAltitude(ReadOnlySpan<char> value)
    {
        if (value.Length < 2 || !int.TryParse(value[1..], out var altitude))
        {
            throw new FormatException($"Invalid altitude format: '{value}'");
        }

        return value[0] switch
        {
            'S' => AltitudeHelper.StandardAltitudesToFlightLevel[altitude * 100],
            'F' => altitude * 100,
            _ => throw new FormatException($"Invalid altitude format: '{value}'"),
        };
    }

    private static int ParseMinimalAltitude(ReadOnlySpan<char> value)
    {
        return value.IsWhiteSpace() ? 0 : int.Parse(value);
    }

    public async Task<IDictionary<string, IDictionary<string, IList<PreferredRoute>>>> GetPreferredRouteWithCacheAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetPreferredRouteWithCacheAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);
            return await GetPreferredRoutesAsync(ct);
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IList<PreferredRoute>> GetPreferredRouteAsync(string departure, string arrival, CancellationToken ct = default)
    {
        var routes = await GetPreferredRouteWithCacheAsync(ct);
        if (routes.TryGetValue(departure, out var arrivalDict)
            && arrivalDict.TryGetValue(arrival, out var preferredRoute))
        {
            return preferredRoute;
        }
        return [];
    }

    public class Option
    {
        public const string LOCATION = "Navdata:S3";

        public required string ServiceUrl { get; set; }
        public required string AccessKey { get; set; }
        public required string SecretKey { get; set; }
        public required string Bucket { get; set; }
        public required string ArincPath { get; set; }
        public required string AipPath { get; set; }
        public required string RoutePath { get; set; }
    }
}
