using System.CommandLine;
using System.CommandLine.Invocation;
using System.Diagnostics;
using System.Text.Json;
using Amazon.Runtime;
using Amazon.S3;
using Arinc424;
using Dapper;
using Microsoft.Data.Sqlite;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;
using nietras.SeparatedValues;

namespace Net.Vatprc.Uniapi;

public class NavdataCommand : Command
{
    private const string NAVDATA_BUCKET = "navdata";
    private const string FILE_BUCKET = "vatprc-files";
    private const string ROUTE_KEY = "sectors/Route-Server.csv";

    protected readonly WebApplication App;

    private readonly Option<string> ServiceUrl = new("--service-url");
    private readonly Option<string> AccessKey = new("--access-key");
    private readonly Option<string> SecretKey = new("--secret-key");
    private readonly Option<string> ArincPath = new("--arinc", () => "cesfpl.pc");
    private readonly Option<string> ArincLocalPath = new("--arinc-local", () => "../Data/cesfpl.pc");
    private readonly Option<string> AipPath = new("--aip", () => "aip.db3");
    private readonly Option<string> AipLocalPath = new("--aip-local", () => "../Data/aip.db3");

    public NavdataCommand(WebApplication app) : base("navdata", "Import navdata")
    {
        App = app;
        this.SetHandler(Handle);
        AddOption(ServiceUrl);
        AddOption(AccessKey);
        AddOption(SecretKey);
        AddOption(ArincPath);
        AddOption(ArincLocalPath);
        AddOption(AipPath);
        AddOption(AipLocalPath);
    }

    protected async Task<Data424> GetArincFileAsync(
        string serviceUrl,
        string accessKey,
        string secretKey,
        string bucket,
        string arincPath,
        string arincLocalPath)
    {
        if (!File.Exists(arincLocalPath))
        {
            var credentials = new BasicAWSCredentials(accessKey, secretKey);
            var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
            {
                ServiceURL = serviceUrl,
            });
            var response = await s3Client.GetObjectAsync(bucket, arincPath);
            await response.WriteResponseStreamToFileAsync(arincLocalPath, false, default);
        }

        var strings = File.ReadAllLines(arincLocalPath)
            .Select(line => line.Trim())
            .Where(line => !string.IsNullOrWhiteSpace(line));

        var meta = Meta424.Create(Supplement.V18);
        var data = Data424.Create(meta, strings, out var invalid, out var skipped);
        return data;
    }

    protected async Task<SqliteConnection> GetAipDataSync(
        string serviceUrl,
        string accessKey,
        string secretKey,
        string bucket,
        string aipPath,
        string aipLocalPath)
    {
        if (!File.Exists(aipLocalPath))
        {
            var credentials = new BasicAWSCredentials(accessKey, secretKey);
            var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
            {
                ServiceURL = serviceUrl,
            });
            var response = await s3Client.GetObjectAsync(bucket, aipPath);
            await response.WriteResponseStreamToFileAsync(aipLocalPath, false, default);
        }
        var aipConnection = new SqliteConnection($"Data Source={aipLocalPath}");
        await aipConnection.OpenAsync();
        return aipConnection;
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

    protected async Task<IEnumerable<RouteData>> GetRouteDataAsync(
        string serviceUrl,
        string accessKey,
        string secretKey,
        string bucket,
        string routeKey,
        string routeLocalPath)
    {
        if (!File.Exists(routeLocalPath))
        {
            var credentials = new BasicAWSCredentials(accessKey, secretKey);
            var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
            {
                ServiceURL = serviceUrl,
            });
            var response = await s3Client.GetObjectAsync(bucket, routeKey);
            await response.WriteResponseStreamToFileAsync(routeLocalPath, false, default);
        }

        using var reader = Sep.Reader().FromFile(routeLocalPath);
        var list = reader.ParallelEnumerate(row =>
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
        return list;
    }

    protected void BuildArincData(
        Data424 arinc,
        IList<Airport> airports,
        IList<AirportGate> gates,
        IList<Airway> airways,
        IList<AirwayFix> airwayFixes,
        IList<NdbNavaid> ndbNavaids,
        IList<PreferredRoute> routes,
        IList<Procedure> procedures,
        IList<Runway> runways,
        IList<VhfNavaid> vhfNavaids,
        IList<Waypoint> waypoints)
    {
        foreach (var airportData in arinc.Airports)
        {
            var airport = new Airport
            {
                Identifier = airportData.Identifier,
                Latitude = airportData.Coordinates.Latitude,
                Longitude = airportData.Coordinates.Longitude,
                Elevation = airportData.Elevation
            };

            airports.Add(airport);

            foreach (var gate in airportData.Gates ?? [])
            {
                var airportGate = new AirportGate
                {
                    Identifier = gate.Identifier,
                    Airport = airport,
                    Latitude = gate.Coordinates.Latitude,
                    Longitude = gate.Coordinates.Longitude
                };
                gates.Add(airportGate);
            }

            foreach (var runwayData in airportData.Thresholds ?? [])
            {
                var runway = new Runway
                {
                    Airport = airport,
                    Identifier = runwayData.Identifier.StripPrefix("RW"),
                    Latitude = runwayData.Coordinates.Latitude,
                    Longitude = runwayData.Coordinates.Longitude
                };
                runways.Add(runway);
            }

            foreach (var procedureData in airportData.Departures ?? [])
            {
                var procedure = new Procedure
                {
                    Airport = airport,
                    Identifier = procedureData.Identifier,
                    SubsectionCode = 'D', // SID
                };
                procedures.Add(procedure);
            }

            foreach (var procedureData in airportData.Arrivals ?? [])
            {
                var procedure = new Procedure
                {
                    Airport = airport,
                    Identifier = procedureData.Identifier,
                    SubsectionCode = 'E', // STAR
                };
                procedures.Add(procedure);
            }
        }

        foreach (var airwayData in arinc.Airways)
        {
            var airway = new Airway
            {
                Identifier = airwayData.Identifier,
            };

            airways.Add(airway);

            foreach (var fixData in airwayData.Sequence ?? [])
            {
                var fix = new AirwayFix
                {
                    Airway = airway,
                    SequenceNumber = (uint)fixData.SeqNumber,
                    FixIdentifier = fixData.Fix.Identifier,
                    FixIcaoCode = fixData.Fix.Icao.ToString(),
                    DirectionalRestriction = fixData.Restriction switch
                    {
                        Arinc424.Routing.Terms.AirwayRestriction.None => ' ',
                        Arinc424.Routing.Terms.AirwayRestriction.Forward => 'F',
                        Arinc424.Routing.Terms.AirwayRestriction.Backward => 'B',
                        _ => throw new ArgumentOutOfRangeException(nameof(fixData.Restriction), fixData.Restriction, null)
                    }
                };
                airwayFixes.Add(fix);

                if (fixData.Descriptions.HasFlag(Arinc424.Waypoints.Terms.WaypointDescriptions.ContinuousSegmentEnd))
                {
                    airway = new Airway
                    {
                        Identifier = airwayData.Identifier,
                    };
                    airways.Add(airway);
                }
            }
        }

        foreach (var ndbData in arinc.Nondirectionals)
        {
            var ndbNavaid = new NdbNavaid
            {
                SectionCode = "DB",
                IcaoCode = ndbData.Icao.ToString(),
                Identifier = ndbData.Identifier,
                Latitude = ndbData.Coordinates.Latitude,
                Longitude = ndbData.Coordinates.Longitude,
            };
            ndbNavaids.Add(ndbNavaid);
        }

        foreach (var vhfData in arinc.Omnidirectionals)
        {
            var vhfNavaid = new VhfNavaid
            {
                IcaoCode = vhfData.Icao.ToString(),
                VorIdentifier = vhfData.Identifier,
                VorLatitude = vhfData.Coordinates.Latitude,
                VorLongitude = vhfData.Coordinates.Longitude,
                DmeIdentifier = vhfData.EquipmentIdentifier,
                DmeLatitude = vhfData.EquipmentCoordinates?.Latitude,
                DmeLongitude = vhfData.EquipmentCoordinates?.Longitude,
            };
            vhfNavaids.Add(vhfNavaid);
        }

        foreach (var waypointData in arinc.EnrouteWaypoints)
        {
            var waypoint = new Waypoint
            {
                SectionCode = "EA",
                RegionCode = "ENRT",
                Identifier = waypointData.Identifier,
                Latitude = waypointData.Coordinates.Latitude,
                Longitude = waypointData.Coordinates.Longitude,
                IcaoCode = waypointData.Icao.ToString(),
            };
            waypoints.Add(waypoint);
        }
    }

    protected async Task Handle(InvocationContext context)
    {
        var serviceUrl = context.ParseResult.GetValueForOption(ServiceUrl) ?? throw new ArgumentNullException(nameof(ServiceUrl));
        var accessKey = context.ParseResult.GetValueForOption(AccessKey) ?? throw new ArgumentNullException(nameof(AccessKey));
        var secretKey = context.ParseResult.GetValueForOption(SecretKey) ?? throw new ArgumentNullException(nameof(SecretKey));
        var arincPath = context.ParseResult.GetValueForOption(ArincPath) ?? throw new ArgumentNullException(nameof(ArincPath));
        var arincLocalPath = context.ParseResult.GetValueForOption(ArincLocalPath) ?? throw new ArgumentNullException(nameof(ArincLocalPath));
        var aipPath = context.ParseResult.GetValueForOption(AipPath) ?? throw new ArgumentNullException(nameof(AipPath));
        var aipLocalPath = context.ParseResult.GetValueForOption(AipLocalPath) ?? throw new ArgumentNullException(nameof(AipLocalPath));

        using var scope = App.Services.CreateScope();
        using var db = scope.ServiceProvider.GetRequiredService<VATPRCContext>();

        await db.Airport.ExecuteDeleteAsync();
        await db.AirportGate.ExecuteDeleteAsync();
        await db.Airway.ExecuteDeleteAsync();
        await db.AirwayFix.ExecuteDeleteAsync();
        await db.NdbNavaid.ExecuteDeleteAsync();
        await db.PreferredRoute.ExecuteDeleteAsync();
        await db.Procedure.ExecuteDeleteAsync();
        await db.Runway.ExecuteDeleteAsync();
        await db.VhfNavaid.ExecuteDeleteAsync();
        await db.Waypoint.ExecuteDeleteAsync();

        var airports = new List<Airport>();
        var gates = new List<AirportGate>();
        var airways = new List<Airway>();
        var airwayFixes = new List<AirwayFix>();
        var ndbNavaids = new List<NdbNavaid>();
        var routes = new List<PreferredRoute>();
        var procedures = new List<Procedure>();
        var runways = new List<Runway>();
        var vhfNavaids = new List<VhfNavaid>();
        var waypoints = new List<Waypoint>();

        var airwayFixesRaw = new Dictionary<string, IList<string>>();
        var existingAirwaySegments = new HashSet<string>();

        var arinc = await GetArincFileAsync(serviceUrl, accessKey, secretKey, NAVDATA_BUCKET, arincPath, arincLocalPath);

        BuildArincData(
            arinc,
            airports,
            gates,
            airways,
            airwayFixes,
            ndbNavaids,
            routes,
            procedures,
            runways,
            vhfNavaids,
            waypoints);

        // TODO: AIP

        var route = await GetRouteDataAsync(
            serviceUrl,
            accessKey,
            secretKey,
            FILE_BUCKET,
            ROUTE_KEY,
            "../Data/Route-Server.csv");
        foreach (var routeData in route)
        {
            Console.WriteLine($"Processing route: {JsonSerializer.Serialize(routeData)}");
            routes.Add(new PreferredRoute
            {
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
        }

        db.Airport.AddRange(airports);
        db.AirportGate.AddRange(gates);
        db.Runway.AddRange(runways);
        db.Procedure.AddRange(procedures);
        db.Airway.AddRange(airways);
        db.AirwayFix.AddRange(airwayFixes);
        db.NdbNavaid.AddRange(ndbNavaids);
        db.VhfNavaid.AddRange(vhfNavaids);
        db.Waypoint.AddRange(waypoints);
        db.PreferredRoute.AddRange(routes);

        await db.SaveChangesAsync();
    }
}
