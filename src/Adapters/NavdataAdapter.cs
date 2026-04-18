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

    protected record RouteData
    {
        public required string Dep;
        public required string Arr;
        public required string Name;
        public required string EvenOdd;
        public required string AltList;
        public required string MinAlt;
        public required string Route;
        public required string Remarks;
    }

    public async Task<IDictionary<string, IDictionary<string, IList<PreferredRoute>>>> GetPreferredRoutesAsync(CancellationToken ct = default)
    {
        var routesText = await GetRoutesAsync(ct);
        using var reader = Sep.Reader().FromText(routesText);
        var route = reader.ParallelEnumerate(row =>
        {
            return new RouteData
            {
                Dep = row["Dep"].Parse<string>(),
                Arr = row["Arr"].Parse<string>(),
                Name = row["Name"].Parse<string>(),
                EvenOdd = row["EvenOdd"].Parse<string>(),
                AltList = row["AltList"].Parse<string>(),
                MinAlt = row["MinAlt"].Parse<string>(),
                Route = row["Route"].Parse<string>(),
                Remarks = row["Remarks"].Parse<string>(),
            };
        }).ToList();
        var routes = route.Select(routeData => new PreferredRoute
        {
            Id = Ulid.NewUlid(),
            Departure = routeData.Dep,
            Arrival = routeData.Arr,
            RawRoute = routeData.Route,
            CruisingLevelRestriction = routeData.EvenOdd switch
            {
                "SE" => PreferredRoute.LevelRestrictionType.StandardEven,
                "SO" => PreferredRoute.LevelRestrictionType.StandardOdd,
                "FE" => PreferredRoute.LevelRestrictionType.FlightLevelEven,
                "FO" => PreferredRoute.LevelRestrictionType.FlightLevelOdd,
                _ => PreferredRoute.LevelRestrictionType.Standard,
            },
            AllowedAltitudes = routeData.AltList.Split('/').Where(s => !string.IsNullOrWhiteSpace(s)).Select(alt =>
            {
                if (alt.StartsWith('S') && int.TryParse(alt[1..], out var standardAltitude))
                {
                    return AltitudeHelper.StandardAltitudesToFlightLevel[standardAltitude * 100];
                }
                else if (alt.StartsWith('F') && int.TryParse(alt[1..], out var flightLevel))
                {
                    return flightLevel * 100;
                }
                else
                {
                    throw new FormatException($"Invalid altitude format: '{alt}'");
                }
            }).ToList(),
            MinimalAltitude = !string.IsNullOrWhiteSpace(routeData.MinAlt) ? int.Parse(routeData.MinAlt) : 0,
            Remarks = routeData.Remarks,
        });
        return (IDictionary<string, IDictionary<string, IList<PreferredRoute>>>)routes
            .GroupBy(r => r.Departure)
            .ToDictionary(
                g => g.Key,
                g => g
                    .GroupBy(r => r.Arrival)
                    .ToDictionary(g2 => g2.Key, g2 => g2.First()));
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
