using Amazon.Runtime;
using Amazon.S3;
using System.Globalization;
using Microsoft.Extensions.Caching.Memory;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Utils;
using nietras.SeparatedValues;

namespace Net.Vatprc.Uniapi.Adapters;

public class NavadataAdapter(IMemoryCache cache, IOptions<NavadataAdapter.Option> options)
{
    protected record AirportData
    {
        public required string Id;
        public required string Identifier;
        public required double Latitude;
        public required double Longitude;
    }

    protected record WaypointData
    {
        public required string IcaoCode;
        public required string Identifier;
        public required double Latitude;
        public required double Longitude;
    }

    protected record VhfNavaidData
    {
        public required string IcaoCode;
        public required string Identifier;
        public required double? Latitude;
        public required double? Longitude;
    }

    protected record NdbNavaidData
    {
        public required string IcaoCode;
        public required string Identifier;
        public required double Latitude;
        public required double Longitude;
    }

    protected record AirwayData
    {
        public required string Id;
        public required string Identifier;
    }

    protected record AirwayFixData
    {
        public required string AirwayId;
        public required long SequenceNumber;
        public required string FixIdentifier;
        public required string FixIcaoCode;
        public required char DirectionalRestriction;
    }

    protected record ProcedureData
    {
        public required string Identifier;
        public required string AirportId;
        public required char SubsectionCode;
    }

    public async Task<string> GetArincData(CancellationToken ct = default)
    {
        return await GetS3FileAsync(options.Value.ArincPath, ct);
    }

    public async Task<string> GetRoutesAsync(CancellationToken ct = default)
    {
        return await GetS3FileAsync(options.Value.RoutePath, ct);
    }

    public async Task<string> GetModelCsvAsync(string modelName, CancellationToken ct = default)
    {
        var prefix = options.Value.AipPath.TrimEnd('/');
        return await GetS3FileAsync($"{prefix}/{modelName}.csv", ct);
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

    public async Task<IReadOnlyDictionary<string, Airport>> GetAirportsAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetAirportsAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);
            var csv = await GetModelCsvAsync("airport", ct);
            using var reader = Sep.Reader().FromText(csv);
            var airports = reader.ParallelEnumerate(row => new AirportData
            {
                Id = row["id"].Parse<string>(),
                Identifier = row["identifier"].Parse<string>(),
                Latitude = row["latitude"].Parse<double>(),
                Longitude = row["longitude"].Parse<double>(),
            }).ToDictionary(
                airport => airport.Identifier,
                airport => new Airport(airport.Identifier, airport.Identifier, airport.Latitude, airport.Longitude),
                StringComparer.OrdinalIgnoreCase);
            return (IReadOnlyDictionary<string, Airport>)airports;
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IReadOnlyDictionary<string, IReadOnlyList<Waypoint>>> GetWaypointsAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetWaypointsAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);
            var csv = await GetModelCsvAsync("waypoint", ct);
            using var reader = Sep.Reader().FromText(csv);
            var waypoints = reader.ParallelEnumerate(row => new WaypointData
            {
                IcaoCode = row["icao_code"].Parse<string>(),
                Identifier = row["identifier"].Parse<string>(),
                Latitude = row["latitude"].Parse<double>(),
                Longitude = row["longitude"].Parse<double>(),
            }).Select(waypoint => new Waypoint(
                waypoint.IcaoCode,
                waypoint.Identifier,
                waypoint.Latitude,
                waypoint.Longitude));

            return (IReadOnlyDictionary<string, IReadOnlyList<Waypoint>>)waypoints
                .GroupBy(waypoint => waypoint.Identifier, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(
                    group => group.Key,
                    group => (IReadOnlyList<Waypoint>)group.ToList(),
                    StringComparer.OrdinalIgnoreCase);
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IReadOnlyDictionary<string, IReadOnlyList<VhfNavaid>>> GetVhfNavaidsAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetVhfNavaidsAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);
            var csv = await GetModelCsvAsync("vhf_navaid", ct);
            using var reader = Sep.Reader().FromText(csv);
            var navaids = reader.ParallelEnumerate(row => new VhfNavaidData
            {
                IcaoCode = row["icao_code"].Parse<string>(),
                Identifier = row["vor_identifier"].Parse<string>(),
                Latitude = ParseNullableDouble(row["vor_latitude"].Parse<string>()),
                Longitude = ParseNullableDouble(row["vor_longitude"].Parse<string>()),
            })
            .Where(navaid => navaid.Latitude != null && navaid.Longitude != null)
            .Select(navaid => new VhfNavaid(
                navaid.IcaoCode,
                navaid.Identifier,
                navaid.Latitude!.Value,
                navaid.Longitude!.Value));

            return (IReadOnlyDictionary<string, IReadOnlyList<VhfNavaid>>)navaids
                .GroupBy(navaid => navaid.Identifier, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(
                    group => group.Key,
                    group => (IReadOnlyList<VhfNavaid>)group.ToList(),
                    StringComparer.OrdinalIgnoreCase);
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IReadOnlyDictionary<string, IReadOnlyList<NdbNavaid>>> GetNdbNavaidsAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetNdbNavaidsAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);
            var csv = await GetModelCsvAsync("ndb_navaid", ct);
            using var reader = Sep.Reader().FromText(csv);
            var navaids = reader.ParallelEnumerate(row => new NdbNavaidData
            {
                IcaoCode = row["icao_code"].Parse<string>(),
                Identifier = row["identifier"].Parse<string>(),
                Latitude = row["latitude"].Parse<double>(),
                Longitude = row["longitude"].Parse<double>(),
            }).Select(navaid => new NdbNavaid(
                navaid.IcaoCode,
                navaid.Identifier,
                navaid.Latitude,
                navaid.Longitude));

            return (IReadOnlyDictionary<string, IReadOnlyList<NdbNavaid>>)navaids
                .GroupBy(navaid => navaid.Identifier, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(
                    group => group.Key,
                    group => (IReadOnlyList<NdbNavaid>)group.ToList(),
                    StringComparer.OrdinalIgnoreCase);
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IReadOnlyDictionary<string, IReadOnlyList<AirwayLeg>>> GetAirwayLegLookupAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetAirwayLegLookupAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);

            var airwayCsv = await GetModelCsvAsync("airway", ct);
            var airwayFixCsv = await GetModelCsvAsync("airway_fix", ct);

            using var airwayReader = Sep.Reader().FromText(airwayCsv);
            using var airwayFixReader = Sep.Reader().FromText(airwayFixCsv);

            var airways = airwayReader.ParallelEnumerate(row => new AirwayData
            {
                Id = row["id"].Parse<string>(),
                Identifier = row["identifier"].Parse<string>(),
            }).ToDictionary(airway => airway.Id, StringComparer.OrdinalIgnoreCase);

            var fixes = await GetFixLookupByIdentifierAsync(ct);

            var airwayFixes = airwayFixReader.ParallelEnumerate(row => new AirwayFixData
            {
                AirwayId = row["airway_id"].Parse<string>(),
                SequenceNumber = row["sequence_number"].Parse<long>(),
                FixIdentifier = row["fix_identifier"].Parse<string>(),
                FixIcaoCode = row["fix_icao_code"].Parse<string>(),
                DirectionalRestriction = row["directional_restriction"].Parse<char>(),
            }).ToList();

            var lookup = airwayFixes
                .GroupBy(airwayFix => airwayFix.AirwayId, StringComparer.OrdinalIgnoreCase)
                .Where(group => airways.ContainsKey(group.Key))
                .ToDictionary(
                    group => airways[group.Key].Identifier,
                    group =>
                    {
                        var sequence = group.OrderBy(airwayFix => airwayFix.SequenceNumber).ToList();
                        var legs = new List<AirwayLeg>();
                        for (var i = 0; i < sequence.Count - 1; i++)
                        {
                            var from = ResolveFix(fixes, sequence[i].FixIdentifier, sequence[i].FixIcaoCode);
                            var to = ResolveFix(fixes, sequence[i + 1].FixIdentifier, sequence[i + 1].FixIcaoCode);
                            if (from == null || to == null)
                            {
                                continue;
                            }

                            legs.Add(new AirwayLeg(
                                from,
                                to,
                                airways[group.Key].Identifier,
                                sequence[i].DirectionalRestriction switch
                                {
                                    'F' => AirwayLeg.AirwayDirection.FORWARD,
                                    'B' => AirwayLeg.AirwayDirection.BACKWARD,
                                    _ => AirwayLeg.AirwayDirection.BOTH,
                                }));
                        }
                        return (IReadOnlyList<AirwayLeg>)legs;
                    },
                    StringComparer.OrdinalIgnoreCase);

            return (IReadOnlyDictionary<string, IReadOnlyList<AirwayLeg>>)lookup;
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    public async Task<IReadOnlyDictionary<string, IReadOnlyDictionary<string, Procedure>>> GetProceduresByAirportAsync(
        char subsectionCode,
        CancellationToken ct = default)
    {
        var cacheKey = $"{nameof(NavadataAdapter)}#{nameof(GetProceduresByAirportAsync)}#{subsectionCode}";
        return await cache.GetOrCreateAsync(cacheKey, async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);

            var airportsCsv = await GetModelCsvAsync("airport", ct);
            var procedureCsv = await GetModelCsvAsync("procedure", ct);

            using var airportReader = Sep.Reader().FromText(airportsCsv);
            using var procedureReader = Sep.Reader().FromText(procedureCsv);

            var airports = airportReader.ParallelEnumerate(row => new AirportData
            {
                Id = row["id"].Parse<string>(),
                Identifier = row["identifier"].Parse<string>(),
                Latitude = row["latitude"].Parse<double>(),
                Longitude = row["longitude"].Parse<double>(),
            }).ToDictionary(airport => airport.Id, airport => airport.Identifier, StringComparer.OrdinalIgnoreCase);

            var procedures = procedureReader.ParallelEnumerate(row => new ProcedureData
            {
                Identifier = row["identifier"].Parse<string>(),
                AirportId = row["airport_id"].Parse<string>(),
                SubsectionCode = row["subsection_code"].Parse<char>(),
            })
            .Where(procedure => char.ToUpperInvariant(procedure.SubsectionCode) == char.ToUpperInvariant(subsectionCode))
            .Where(procedure => airports.ContainsKey(procedure.AirportId))
            .GroupBy(procedure => airports[procedure.AirportId], StringComparer.OrdinalIgnoreCase)
            .ToDictionary(
                group => group.Key,
                group => (IReadOnlyDictionary<string, Procedure>)group
                    .GroupBy(procedure => procedure.Identifier, StringComparer.OrdinalIgnoreCase)
                    .ToDictionary(
                        procedureGroup => procedureGroup.Key,
                        procedureGroup => new Procedure(procedureGroup.Key, []),
                        StringComparer.OrdinalIgnoreCase),
                StringComparer.OrdinalIgnoreCase);

            return (IReadOnlyDictionary<string, IReadOnlyDictionary<string, Procedure>>)procedures;
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    protected async Task<IReadOnlyDictionary<string, IReadOnlyList<FixWithIdentifier>>> GetFixLookupByIdentifierAsync(CancellationToken ct = default)
    {
        return await cache.GetOrCreateAsync($"{nameof(NavadataAdapter)}#{nameof(GetFixLookupByIdentifierAsync)}", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromHours(1);

            var airports = await GetAirportsAsync(ct);
            var waypoints = await GetWaypointsAsync(ct);
            var vhfNavaids = await GetVhfNavaidsAsync(ct);
            var ndbNavaids = await GetNdbNavaidsAsync(ct);

            var allFixes = airports.Values.Cast<FixWithIdentifier>()
                .Concat(waypoints.Values.SelectMany(group => group))
                .Concat(vhfNavaids.Values.SelectMany(group => group))
                .Concat(ndbNavaids.Values.SelectMany(group => group));

            return (IReadOnlyDictionary<string, IReadOnlyList<FixWithIdentifier>>)allFixes
                .GroupBy(fix => fix.Identifier, StringComparer.OrdinalIgnoreCase)
                .ToDictionary(
                    group => group.Key,
                    group => (IReadOnlyList<FixWithIdentifier>)group.ToList(),
                    StringComparer.OrdinalIgnoreCase);
        }) ?? throw new InvalidOperationException("Cache entry factory returned null");
    }

    protected static FixWithIdentifier? ResolveFix(
        IReadOnlyDictionary<string, IReadOnlyList<FixWithIdentifier>> fixes,
        string identifier,
        string icaoCode)
    {
        if (!fixes.TryGetValue(identifier, out var matches))
        {
            return null;
        }

        return matches.FirstOrDefault(fix => string.Equals(fix.IcaoCode, icaoCode, StringComparison.OrdinalIgnoreCase))
            ?? matches.FirstOrDefault();
    }

    protected static double? ParseNullableDouble(string value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return null;
        }

        return double.Parse(value, CultureInfo.InvariantCulture);
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
