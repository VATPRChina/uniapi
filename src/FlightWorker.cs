using System.Diagnostics;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi;

public class FlightWorker(
    ILogger<FlightWorker> Logger,
    IOptionsMonitor<FlightWorker.Option> Options,
    VatsimAdapter VatsimService,
    IServiceScopeFactory ScopeFactory,
    ActivitySource ActivitySource
) : BackgroundService
{
    protected readonly PeriodicTimer Timer = new(TimeSpan.FromSeconds(Options.CurrentValue.PeriodInSeconds));

    protected override async Task ExecuteAsync(CancellationToken ct)
    {
        if (!Options.CurrentValue.Enabled)
        {
            Logger.LogInformation("FlightWorker is disabled.");
            return;
        }
        Logger.LogInformation("FlightWorker is starting at interval {Interval}s.", Timer.Period.TotalSeconds);
        do
        {
            using var activity = ActivitySource.StartActivity("FlightWorker.RunAsync");

            var startTime = DateTimeOffset.Now;
            Logger.LogInformation("Running FlightWorker at {Time}.", startTime);
            try
            {
                await RunAsync(ct);
            }
            catch (Exception e)
            {
                Logger.LogError(e, "An error occurred while running FlightWorker at {Time}.", startTime);
                activity?.SetStatus(ActivityStatusCode.Error, e.Message);
            }
        } while (!ct.IsCancellationRequested && await Timer.WaitForNextTickAsync(ct));
        Logger.LogInformation("FlightWorker is stopping.");
    }

    protected bool IsChinaAirport(string ident)
    {
        return ident.Length == 4 && ident[0] == 'Z' && (
            ident[1] == 'B' ||
            ident[1] == 'M' ||
            ident[1] == 'S' ||
            ident[1] == 'P' ||
            ident[1] == 'G' ||
            ident[1] == 'J' ||
            ident[1] == 'Y' ||
            ident[1] == 'W' ||
            ident[1] == 'L' ||
            ident[1] == 'U' ||
            ident[1] == 'H'
        );
    }

    protected async ValueTask RunAsync(CancellationToken ct)
    {
        var scope = ScopeFactory.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<Database>();

        var data = await VatsimService.GetOnlineData(ct);

        await ClearFinalizedFlights(db, data, ct);
        foreach (var pilot in data.Pilots)
        {
            using var activity = ActivitySource.StartActivity("FlightWorker.UpdateFlight");
            try
            {
                await UpdateFlight(db, pilot, ct);
            }
            catch (Exception e)
            {
                Logger.LogError(e, "Failed to update flight for pilot {Callsign}.", pilot.Callsign);
                activity?.SetStatus(ActivityStatusCode.Error, e.Message);
            }
        }
    }

    protected async ValueTask ClearFinalizedFlights(Database db, Adapters.VatsimAdapterModels.VatsimData data, CancellationToken ct = default)
    {
        var flights = await db.Flight
            .Where(f => f.FinalizedAt == null).ToArrayAsync(ct);

        var finalized = flights
            .Where(f => !data.Pilots.Any(p => p.Callsign == f.Callsign && p.Cid.ToString() == f.Cid) &&
                DateTimeOffset.UtcNow - f.LastObservedAt > TimeSpan.FromMinutes(Options.CurrentValue.TimeoutInMinutes))
            .ToArray();

        foreach (var flight in finalized)
        {
            flight.FinalizedAt = flight.LastObservedAt;
        }
        await db.SaveChangesAsync(ct);
    }

    protected async ValueTask UpdateFlight(Database db, Adapters.VatsimAdapterModels.Pilot pilot, CancellationToken ct = default)
    {
        if (pilot.FlightPlan == null)
        {
            Logger.LogDebug("Ignore {Callsign} with no flight plan.", pilot.Callsign);
            return;
        }
        bool isOverflyChina = false;
        if (!IsChinaAirport(pilot.FlightPlan.Departure) && !IsChinaAirport(pilot.FlightPlan.Arrival))
        {
            if (!isOverflyChina)
            {
                Logger.LogDebug("Ignore {Callsign} ({Departure}-{Arrival}) with no VATPRC airport.",
                    pilot.Callsign, pilot.FlightPlan.Departure, pilot.FlightPlan.Arrival);
                return;
            }
        }
        if (pilot.FlightPlan.Departure == pilot.FlightPlan.Arrival)
        {
            Logger.LogDebug("Ignore {Callsign} with same departure and arrival airport.", pilot.Callsign);
            return;
        }
        if (pilot.FlightPlan.FlightRules != "I")
        {
            Logger.LogDebug("Ignore {Callsign} with flight rules {FlightRules} (not IFR).", pilot.Callsign, pilot.FlightPlan.FlightRules);
            return;
        }
        Logger.LogDebug("Discovered flight: {Callsign}", pilot.Callsign);

        var flight = await db.Flight
            .FirstOrDefaultAsync(f =>
                f.Callsign == pilot.Callsign &&
                f.Cid == pilot.Cid.ToString() &&
                f.FinalizedAt == null,
            ct);

        if (flight != null && (flight.Departure != pilot.FlightPlan.Departure || flight.Arrival != pilot.FlightPlan.Arrival))
        {
            flight.FinalizedAt = flight.LastObservedAt;
            flight = null;
        }
        if (flight == null)
        {
            flight = new Flight
            {
                Id = Ulid.NewUlid(),
                State = Flight.FlightState.UNKNOWN
            };
            db.Flight.Add(flight);
        }

        flight.Cid = pilot.Cid.ToString();
        flight.Callsign = pilot.Callsign;
        flight.LastObservedAt = pilot.LastUpdated.ToUniversalTime();
        flight.Latitude = pilot.Latitude;
        flight.Longitude = pilot.Longitude;
        flight.Altitude = pilot.Altitude;
        flight.Departure = pilot.FlightPlan.Departure;
        flight.Arrival = pilot.FlightPlan.Arrival;
        flight.CruiseTas = uint.Parse(pilot.FlightPlan.CruiseTas);
        flight.CruisingLevel = ParseFlightAltitude(pilot.FlightPlan.Altitude);
        flight.RawRoute = pilot.FlightPlan.Route;
        var aircraft = FlightPlanUtils.ParseIcaoAircraftCode(pilot.FlightPlan.Aircraft, pilot.FlightPlan.Remarks);
        flight.Aircraft = aircraft.AircraftCode;
        flight.Equipment = aircraft.Equipment;
        flight.Transponder = aircraft.Transponder;
        flight.NavigationPerformance = aircraft.NavigationPerformance;

        await db.SaveChangesAsync(ct);
        Logger.LogDebug("Saved flight to database: {Callsign}", pilot.Callsign);
    }

    protected long ParseFlightAltitude(string altitude)
    {
        if (long.TryParse(altitude, out var result))
        {
            return result;
        }
        if (altitude.StartsWith("FL", StringComparison.OrdinalIgnoreCase))
        {
            if (long.TryParse(altitude[2..], out result))
            {
                return result * 100;
            }
        }
        throw new FormatException($"Invalid altitude format: {altitude}");
    }

    public class Option
    {
        public const string LOCATION = "Worker:Flight";

        public required bool Enabled { get; set; }

        public required uint PeriodInSeconds { get; set; }

        public required uint TimeoutInMinutes { get; set; }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddHostedService<FlightWorker>();
        return builder;
    }
}
