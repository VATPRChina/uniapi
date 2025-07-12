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
    protected readonly WebApplication App;

    private readonly Option<string> ServiceUrl = new("--service-url");
    private readonly Option<string> AccessKey = new("--access-key");
    private readonly Option<string> SecretKey = new("--secret-key");
    private readonly Option<string> Bucket = new("--bucket", () => "navdata");
    private readonly Option<string> ArincPath = new("--arinc", () => "cesfpl.pc");
    private readonly Option<string> AipPath = new("--aip", () => "aip.db3");
    private readonly Option<string> AipLocalPath = new("--aip-local", () => "Data/aip.db3");

    public NavdataCommand(WebApplication app) : base("navdata", "Import navdata")
    {
        App = app;
        this.SetHandler(Handle);
        AddOption(ServiceUrl);
        AddOption(AccessKey);
        AddOption(SecretKey);
        AddOption(Bucket);
        AddOption(ArincPath);
        AddOption(AipPath);
        AddOption(AipLocalPath);
    }

    protected async Task<Data424> GetArincFileAsync(
        string serviceUrl,
        string accessKey,
        string secretKey,
        string bucket,
        string arincPath)
    {
        IEnumerable<string> strings = [];

        if (File.Exists("Data/cesfpl.pc"))
        {
            strings = File.ReadAllLines("Data/cesfpl.pc")
                .Select(line => line.Trim())
                .Where(line => !string.IsNullOrWhiteSpace(line));
        }
        else
        {
            var credentials = new BasicAWSCredentials(accessKey, secretKey);
            var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
            {
                ServiceURL = serviceUrl,
            });
            var response = await s3Client.GetObjectAsync(bucket, arincPath);
            using var reader = new StreamReader(response.ResponseStream);
            var content = await reader.ReadToEndAsync();
            Console.WriteLine($"Read {content.Length} characters from {arincPath} in bucket {bucket} at {serviceUrl}");
            strings = content.Split('\n')
               .Select(line => line.Trim())
               .Where(line => !string.IsNullOrWhiteSpace(line))
               .ToList();
        }

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
        var bucket = context.ParseResult.GetValueForOption(Bucket) ?? throw new ArgumentNullException(nameof(Bucket));
        var arincPath = context.ParseResult.GetValueForOption(ArincPath) ?? throw new ArgumentNullException(nameof(ArincPath));
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

        var arinc = await GetArincFileAsync(serviceUrl, accessKey, secretKey, bucket, arincPath);

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

        var airportsIndex = airports.ToDictionary(a => a.Identifier);
        var airportRunwaysIndex = runways.ToDictionary(a => $"{a.AirportIdentifier}/{a.Identifier}");

        using var aipdb = await GetAipDataSync(serviceUrl, accessKey, secretKey, bucket, aipPath, aipLocalPath);

        var airportList = await aipdb.QueryAsync("SELECT * FROM AD_HP WHERE CHINA = 'Y';");
        foreach (var airportData in airportList)
        {
            var airport = airportsIndex.GetValueOrDefault((string)airportData.CODE_ICAO);
            if (airport == null)
            {
                airport = new Airport
                {
                    Identifier = "#" + airportData.CODE_ICAO,
                    Latitude = Geography.ParseCaacCoordinate(airportData.GEO_LAT),
                    Longitude = Geography.ParseCaacCoordinate(airportData.GEO_LONG),
                    Elevation = (int)airportData.VAL_ELEV,
                };
                airports.Add(airport);
            }

            var runwayList = await aipdb.QueryAsync("SELECT * FROM RWY WHERE CODE_AIRPORT = @Icao;", new { Icao = airportData.CODE_ICAO });
            foreach (var runway in runwayList)
            {
                var directions = ((string)runway.TXT_DESIG).Split('/');
                if (runway.TXT_RMK == null)
                {
                    Console.WriteLine($"Runway {runway.CODE_AIRPORT}/{runway.TXT_DESIG} has no coordinates in TXT_RMK, skipping.");
                    continue;
                }
                var coords = ((string)runway.TXT_RMK).Split('-');
                Debug.Assert(directions.Length == 2 && coords.Length == 2);
                for (var i = 0; i < 2; i++)
                {
                    var ident = $"{runway.CODE_AIRPORT}/{directions[i]}";
                    if (!airportRunwaysIndex.ContainsKey(ident))
                    {
                        var coordsCurrent = coords[i].Split(',');
                        Debug.Assert(coordsCurrent.Length == 2);
                        var coord = runway.TXT_RMK.Split('-');
                        runways.Add(new Runway
                        {
                            Airport = airport,
                            Identifier = "#" + directions[i],
                            Latitude = Geography.ParseCaacCoordinate(coordsCurrent[0]),
                            Longitude = Geography.ParseCaacCoordinate(coordsCurrent[1]),
                        });
                    }
                }
            }
        }

        // TODO: AIP gates
        // TODO: AIP procedures

        var ndbIndex = ndbNavaids.ToDictionary(n => $"{n.IcaoCode}/{n.Identifier}");
        var ndbList = await aipdb.QueryAsync("""SELECT * FROM NDB WHERE ("CODE_IN_AIRWAY" = 'Y') AND ("CHINA" = 'Y');""");
        foreach (var ndbData in ndbList)
        {
            var ndbNavaid = ndbIndex.GetValueOrDefault($"{ndbData.CODE_AREA}/{ndbData.CODE_ID}");
            if (ndbNavaid == null)
            {
                ndbNavaid = new NdbNavaid
                {
                    SectionCode = "DB",
                    IcaoCode = ndbData.CODE_AREA,
                    Identifier = "#" + ndbData.CODE_ID,
                    Latitude = Geography.ParseCaacCoordinate(ndbData.GEO_LAT),
                    Longitude = Geography.ParseCaacCoordinate(ndbData.GEO_LONG),
                };
                ndbNavaids.Add(ndbNavaid);
            }
        }

        var vhfIndex = vhfNavaids.ToDictionary(v => $"{v.IcaoCode}/{v.VorIdentifier}");
        var vhfList = await aipdb.QueryAsync("""SELECT * FROM VOR WHERE ("CODE_IN_AIRWAY" = 'Y') AND ("CHINA" = 'Y');""");
        foreach (var vhfData in vhfList)
        {
            var vhfNavaid = vhfIndex.GetValueOrDefault($"{vhfData.CODE_AREA}/{vhfData.CODE_ID}");
            if (vhfNavaid == null)
            {
                vhfNavaid = new VhfNavaid
                {
                    IcaoCode = vhfData.CODE_AREA,
                    VorIdentifier = "#" + vhfData.CODE_ID,
                    VorLatitude = Geography.ParseCaacCoordinate(vhfData.GEO_LAT),
                    VorLongitude = Geography.ParseCaacCoordinate(vhfData.GEO_LONG),
                };
                vhfNavaids.Add(vhfNavaid);
            }
        }

        var waypointIndex = waypoints.ToDictionary(w => $"{w.IcaoCode}/{w.Identifier}");
        var waypointList = await aipdb.QueryAsync("""SELECT * FROM DESIGNATED_POINT WHERE "CHINA" = 'Y';""");
        foreach (var waypointData in waypointList)
        {
            var waypoint = waypointIndex.GetValueOrDefault($"{waypointData.CODE_AREA}/{waypointData.CODE_ID}");
            if (waypoint == null)
            {
                waypoint = new Waypoint
                {
                    SectionCode = "EA",
                    RegionCode = "ENRT",
                    Identifier = "#" + waypointData.CODE_ID,
                    Latitude = Geography.ParseCaacCoordinate(waypointData.GEO_LAT),
                    Longitude = Geography.ParseCaacCoordinate(waypointData.GEO_LONG),
                    IcaoCode = waypointData.CODE_AREA,
                };
                waypoints.Add(waypoint);
            }
        }

        // TODO: AIP airways
        // TODO: AIP airwayFixes
        // TODO: AIP preferred routes

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
