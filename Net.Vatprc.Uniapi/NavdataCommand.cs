using System.CommandLine;
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
        await db.Runway.ExecuteDeleteAsync();
        await db.AirportPhysicalRunway.ExecuteDeleteAsync();
        await db.AirportGate.ExecuteDeleteAsync();

        var airports = new List<Models.Navdata.Airport>();
        var runways = new List<Models.Navdata.Runway>();
        var physicalRunways = new List<Models.Navdata.AirportPhysicalRunway>();
        var gates = new List<Models.Navdata.AirportGate>();
        foreach (var line in Sep.Reader().FromFile("2412-data/AD_HP.csv"))
        {
            if (line["VAL_LONG_RWY"].ToString() == "0") continue;
            var airport = new Models.Navdata.Airport
            {
                Identifier = line["CODE_ID"].ToString(),
                Latitude = Geography.ParseCaacCoordinate(line["GEO_LAT_ACCURACY"].ToString()),
                Longitude = Geography.ParseCaacCoordinate(line["GEO_LONG_ACCURACY"].ToString()),
                Elevation = Geography.ConvertMeterToFeet(line["VAL_ELEV"].Parse<int>()),
            };
            airports.Add(airport);
        }
        foreach (var line in File.ReadLines("Runway.txt"))
        {
            // 01 19 40.05888889 116.6176111 40.09286111 116.6122778 ZBAA
            var data = line.Split(' ');
            var airport = airports.FirstOrDefault(a => a.Identifier == data[^1].Trim('"'));
            if (airport == null)
            {
                Console.WriteLine($"Airport {data[0]} not found");
                continue;
            }
            var runway1 = new Models.Navdata.Runway
            {
                Identifier = data[0],
                Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
                Latitude = double.Parse(data[2]),
                Longitude = double.Parse(data[3]),
            };
            var runway2 = new Models.Navdata.Runway
            {
                Identifier = data[1],
                Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
                Latitude = double.Parse(data[4]),
                Longitude = double.Parse(data[5]),
            };
            var physicalRunway = new Models.Navdata.AirportPhysicalRunway
            {
                Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
                Runway1 = runway1,
                Runway2 = runway2,
            };
            runways.Add(runway1);
            runways.Add(runway2);
            physicalRunways.Add(physicalRunway);
        }
        foreach (var line in File.ReadLines("sectors_stand.csv"))
        {
            // "ZBAA","A106","40.082","116.5835","61","",""
            var data = line.Split(',');
            var airport = airports.FirstOrDefault(a => a.Identifier == data[0].Trim('"'));
            if (airport == null)
            {
                Console.WriteLine($"Airport {data[0]} not found");
                continue;
            }
            var gate = new Models.Navdata.AirportGate
            {
                Airport = airports.FirstOrDefault(a => a.Identifier == data[0].Trim('"')),
                Identifier = data[1].Trim('"'),
                Latitude = double.Parse(data[2].Trim('"')),
                Longitude = double.Parse(data[3].Trim('"')),
            };
            gates.Add(gate);
        }

        db.Airport.AddRange(airports);
        db.Runway.AddRange(runways);
        db.AirportPhysicalRunway.AddRange(physicalRunways);
        db.AirportGate.AddRange(gates);
        await db.SaveChangesAsync();
    }
}
