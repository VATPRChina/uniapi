using System.CommandLine;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;
using nietras.SeparatedValues;

namespace Net.Vatprc.Uniapi;

public class NavdataCommand : Command
{
    protected readonly WebApplication App;

    public NavdataCommand(WebApplication app) : base("navdata", "Import navdata")
    {
        App = app;
        this.SetHandler(Handle);
    }

    protected async Task Handle()
    {
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

        var airports = new Dictionary<string, Airport>();
        var gates = new Dictionary<string, AirportGate>();
        var airways = new Dictionary<string, Airway>();
        var airwayFixes = new Dictionary<string, AirwayFix>();
        var ndbNavaids = new Dictionary<string, NdbNavaid>();
        var routes = new Dictionary<string, PreferredRoute>();
        var procedures = new Dictionary<string, Procedure>();
        var runways = new Dictionary<string, Runway>();
        var vhfNavaids = new Dictionary<string, VhfNavaid>();
        var waypoints = new Dictionary<string, Waypoint>();

        var airwayFixesRaw = new Dictionary<string, IList<string>>();
        var existingAirwaySegments = new HashSet<string>();

        await foreach (var record_ in File.ReadLinesAsync("../Data/cesfpl.pc"))
        {
            if (record_ == null) continue;
            var record = " " + record_;
            switch ((record[5], record[6]))
            {
                case ('D', ' '):
                    {
                        if (record[22] != '0' && record[22] != '1') continue; // ignore continuation records
                        var dme = record[52..56].Trim();
                        var vhf = new VhfNavaid
                        {
                            IcaoCode = record[20..22],
                            VorIdentifier = record[14..18].Trim(),
                            VorLatitude = string.IsNullOrWhiteSpace(record[33..42]) ? null : Geography.ParseArincCoordinate(record[33..42]),
                            VorLongitude = string.IsNullOrWhiteSpace(record[42..52]) ? null : Geography.ParseArincCoordinate(record[42..52]),
                            DmeIdentifier = string.IsNullOrEmpty(dme) ? null : dme,
                            DmeLatitude = string.IsNullOrEmpty(dme) ? null : Geography.ParseArincCoordinate(record[56..65]),
                            DmeLongitude = string.IsNullOrEmpty(dme) ? null : Geography.ParseArincCoordinate(record[65..75]),
                        };
                        vhfNavaids.Add($"{vhf.IcaoCode}/{vhf.VorIdentifier}", vhf);
                        break;
                    }
                case ('D', 'B'):
                case ('P', 'N'):
                    {
                        if (record[22] != '0' && record[22] != '1') continue; // ignore continuation records
                        var ndb = new NdbNavaid
                        {
                            SectionCode = record[5..7],
                            AirportIcaoIdent = string.IsNullOrWhiteSpace(record[7..11]) ? null : record[7..11],
                            IcaoCode = record[20..22],
                            Identifier = record[14..18].Trim(),
                            Latitude = Geography.ParseArincCoordinate(record[33..42]),
                            Longitude = Geography.ParseArincCoordinate(record[42..52]),
                        };
                        ndbNavaids.Add($"{ndb.AirportIcaoIdent ?? ndb.IcaoCode}/{ndb.Identifier}", ndb);
                        break;
                    }
                case ('E', 'A'):
                case ('P', 'C'):
                    {
                        if (record[22] != '0' && record[22] != '1') continue; // ignore continuation records
                        var wpt = new Waypoint
                        {
                            SectionCode = record[5..7],
                            RegionCode = record[7..11],
                            IcaoCode = record[20..22],
                            Identifier = record[14..20].Trim(),
                            Latitude = Geography.ParseArincCoordinate(record[33..42]),
                            Longitude = Geography.ParseArincCoordinate(record[42..52]),
                        };
                        waypoints.Add($"{wpt.IcaoCode}/{wpt.Identifier}", wpt);
                        break;
                    }
                case ('E', 'R'):
                    {
                        if (record[39] != '0' && record[39] != '1') continue; // ignore continuation records
                        var ident = record[14..20].Trim();
                        if (!airwayFixesRaw.TryGetValue(ident, out IList<string>? value))
                        {
                            value = [];
                            airwayFixesRaw[ident] = value;
                        }
                        value.Add(record);
                        break;
                    }
                case ('P', ' '):
                    switch (record[13])
                    {
                        case 'A':
                            {
                                if (record[22] != '0' && record[22] != '1') continue; // ignore continuation records
                                var airport = new Airport
                                {
                                    Identifier = record[7..11],
                                    Latitude = Geography.ParseArincCoordinate(record[33..42]),
                                    Longitude = Geography.ParseArincCoordinate(record[42..52]),
                                    Elevation = int.Parse(record[57..62]),
                                };
                                airports.Add(airport.Identifier, airport);
                                break;
                            }
                        case 'B':
                            {
                                Console.WriteLine("Unexpected airport gate record: " + record_);
                                break;
                            }
                        case 'G':
                            {
                                if (record[22] != '0' && record[22] != '1') continue; // ignore continuation records
                                var runway = new Runway
                                {
                                    AirportId = airports[record[7..11]].Id,
                                    Identifier = record[14..19].Trim(),
                                    Latitude = string.IsNullOrWhiteSpace(record[33..42]) ? double.MinValue : Geography.ParseArincCoordinate(record[33..42]),
                                    Longitude = string.IsNullOrWhiteSpace(record[42..52]) ? double.MinValue : Geography.ParseArincCoordinate(record[42..52]),
                                };
                                runways.Add($"{airports[record[7..11]].Identifier}/{runway.Identifier}", runway);
                                break;
                            }
                        case 'D':
                        case 'E':
                        case 'F':
                            {
                                if (record[39] != '0' && record[39] != '1') continue; // ignore continuation records
                                var airportId = record[7..11];
                                var ident = record[14..20].Trim();
                                if (procedures.ContainsKey($"{airportId}/{ident}")) continue;
                                var procedure = new Procedure
                                {
                                    AirportId = airports[airportId].Id,
                                    Identifier = ident,
                                    SubsectionCode = record[13],
                                };
                                procedures.Add($"{airportId}/{ident}", procedure);
                                break;
                            }
                    }
                    break;

                default: continue;
            }
        }

        foreach (var (ident, recordsRaw) in airwayFixesRaw)
        {
            var records = recordsRaw.OrderBy(x => x[2..5]).ThenBy(x => uint.Parse(x[26..30])).ToArray();
            var airway = new Airway
            {
                Identifier = ident.Replace(" ", null),
            };
            foreach (var record in records)
            {
                if (!airways.ContainsKey(airway.Id.ToString())) { airways.Add(airway.Id.ToString(), airway); }
                var fix = new AirwayFix
                {
                    AirwayId = airway.Id,
                    SequenceNumber = uint.Parse(record[26..30]),
                    FixIdentifier = record[30..35].Trim(),
                    FixIcaoCode = record[35..37],
                    DescriptionCode = record[40..44],
                };
                if (fix.DescriptionCode[1] == 'E')
                {
                    airway = new Airway { Identifier = ident.Replace(" ", null) };
                }
                airwayFixes.Add(fix.Id.ToString(), fix);
                existingAirwaySegments.Add($"{airway.Identifier}/{fix.FixIcaoCode}/{fix.FixIdentifier}");
            }
        }

        foreach (var line in Sep.Reader().FromFile("../Data/2412-data/AD_HP.csv"))
        {
            if (line["VAL_LONG_RWY"].ToString() == "0") continue;
            if (airports.ContainsKey(line["CODE_ID"].ToString())) continue;
            var airport = new Airport
            {
                Identifier = line["CODE_ID"].ToString(),
                Latitude = Geography.ParseCaacCoordinate(line["GEO_LAT_ACCURACY"].ToString()),
                Longitude = Geography.ParseCaacCoordinate(line["GEO_LONG_ACCURACY"].ToString()),
                Elevation = Geography.ConvertMeterToFeet(line["VAL_ELEV"].Parse<int>()),
            };
            airports.Add(airport.Identifier, airport);
        }
        foreach (var data in Sep.New(' ').Reader(o => o with { HasHeader = false }).FromFile("../Data/Runway.txt"))
        {
            var airportIdent = data[^1].ToString();
            if (!airports.TryGetValue(airportIdent, out var airport))
            {
                Console.WriteLine($"Runway: Airport {airportIdent} not found");
                continue;
            }
            if (!runways.ContainsKey($"{airport.Identifier}/RW{data[0].ToString()}"))
            {
                var runway1 = new Runway
                {
                    Identifier = data[0].ToString(),
                    AirportId = airport.Id,
                    Latitude = data[2].Parse<double>(),
                    Longitude = data[3].Parse<double>(),
                };
                runways.Add($"{airport.Identifier}/RW{data[0].ToString()}", runway1);
            }
            if (!runways.ContainsKey($"{airport.Identifier}/RW{data[1].ToString()}"))
            {
                var runway2 = new Runway
                {
                    Identifier = data[1].ToString(),
                    AirportId = airport.Id,
                    Latitude = data[4].Parse<double>(),
                    Longitude = data[5].Parse<double>(),
                };
                runways.Add($"{airport.Identifier}/RW{data[1].ToString()}", runway2);
            }
        }
        foreach (var data in Sep.Reader(o => o with { HasHeader = false, Unescape = true }).FromFile("../Data/sectors_stand.csv"))
        {
            var airportIdent = data[0].ToString();
            if (!airports.TryGetValue(airportIdent, out var airport))
            {
                Console.WriteLine($"sectors_stand: Airport {airportIdent} not found");
                continue;
            }
            if (gates.ContainsKey($"{airport.Identifier}/{data[1].ToString()}")) continue;
            var gate = new AirportGate
            {
                AirportId = airport.Id,
                Identifier = data[1].ToString(),
                Latitude = data[2].Parse<double>(),
                Longitude = data[3].Parse<double>(),
            };
            gates.Add($"{airport.Identifier}/{gate.Identifier}", gate);
        }
        var flightAirlinePoints = Sep.Reader().FromFile("../Data/2412-data/FLIGHT_AIRLINE_POINT.csv")
            .Enumerate(p => new
            {
                FlightAirlineId = p["FLIGHT_AIRLINE_ID"].ToString(),
                Sequence = p["Sequnce"].Parse<int>(),
                StartPointIdentifier = !string.IsNullOrEmpty(p["StartPointIdentifier"].ToString()) ? p["StartPointIdentifier"].ToString() : p["StartPointName"].ToString(),
                EndPointIdentifier = !string.IsNullOrEmpty(p["EndPointIdentifier"].ToString()) ? p["EndPointIdentifier"].ToString() : p["EndPointName"].ToString(),
                AirwayName = p["AirwayName"].ToString(),
            })
            .GroupBy(p => p.FlightAirlineId)
            .Select(r => r.OrderBy(p => p.Sequence).ToList())
            .ToList();
        foreach (var line in Sep.Reader().FromFile("../Data/2412-data/FLIGHT_AIRLINE.csv"))
        {
            var airlineId = line["FLIGHT_AIRLINE_ID"].ToString();
            var points = flightAirlinePoints.FirstOrDefault(r => r.First().FlightAirlineId == airlineId);
            if (points == null)
            {
                Console.WriteLine($"Invalid route for {airlineId}");
                continue;
            }
            List<string> routeSegments = [];
            foreach (var point in points)
            {
                if (routeSegments.Count > 0 && routeSegments[^1] == point.StartPointIdentifier)
                {
                    if (routeSegments.Count > 1 && routeSegments[^2] == point.AirwayName)
                    {
                        routeSegments[^1] = point.EndPointIdentifier;
                    }
                    else
                    {
                        routeSegments.AddRange(point.AirwayName, point.EndPointIdentifier);
                    }
                    continue;
                }
                else
                {
                    routeSegments.AddRange(point.StartPointIdentifier, point.AirwayName, point.EndPointIdentifier);
                }
            }
            var route = new PreferredRoute
            {
                Departure = line["StartAirportID"].ToString(),
                Arrival = line["EndAirportID"].ToString(),
                RawRoute = string.Join(" ", routeSegments),
            };
            routes.Add(route.Id.ToString(), route);
        }
        foreach (var data in Sep.New(':').Reader(o => o with { HasHeader = false }).FromFile("../Data/SIDSSTARS.txt"))
        {
            var airportIdent = data[1].ToString();
            if (!airports.TryGetValue(airportIdent, out var airport))
            {
                Console.WriteLine($"SIDSTARS: Airport {airportIdent} not found");
                continue;
            }
            if (!procedures.ContainsKey($"{airport.Identifier}/{data[3].ToString()}"))
            {
                if (data[0].ToString().StartsWith("//")) continue;
                var procedure = new Procedure
                {
                    AirportId = airport.Id,
                    Identifier = data[3].ToString(),
                    SubsectionCode = data[0].ToString() == "SID" ? 'D' : data[0].ToString() == "STAR" ? 'E' : 'X',
                };
                if (procedure.SubsectionCode == 'X') throw new Exception("Unknown procedure type: " + data.ToString());
                procedures.Add($"{airport.Identifier}/{data[3].ToString()}", procedure);
            }
        }

        db.Airport.AddRange(airports.Values);
        db.AirportGate.AddRange(gates.Values);
        db.Airway.AddRange(airways.Values); // TODO: nimbus
        db.AirwayFix.AddRange(airwayFixes.Values); // TODO: nimbus
        db.NdbNavaid.AddRange(ndbNavaids.Values); // TODO: nimbus
        db.PreferredRoute.AddRange(routes.Values);
        db.Procedure.AddRange(procedures.Values);
        db.Runway.AddRange(runways.Values);
        db.VhfNavaid.AddRange(vhfNavaids.Values); // TODO: nimbus
        db.Waypoint.AddRange(waypoints.Values); // TODO: nimbus

        await db.SaveChangesAsync();
    }
}
