using System.Text.RegularExpressions;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services;

public class FlightWorker(
    ILogger<FlightWorker> Logger,
    IOptionsMonitor<FlightWorker.Option> Options,
    VatsimService VatsimService,
    IServiceScopeFactory ScopeFactory
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
            var startTime = DateTimeOffset.Now;
            Logger.LogInformation("Running FlightWorker at {Time}.", startTime);
            try
            {
                await RunAsync(ct);
            }
            catch (Exception e)
            {
                Logger.LogError(e, "An error occurred while running FlightWorker at {Time}.", startTime);
                e.SetSentryMechanism(nameof(FlightWorker), handled: false);
                SentrySdk.CaptureException(e, scope =>
                {
                    scope.TransactionName = $"{nameof(FlightWorker)}@{startTime}";
                });
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
            ident[1] == 'H'
        );
    }

    protected async ValueTask RunAsync(CancellationToken ct)
    {
        var scope = ScopeFactory.CreateScope();
        var db = scope.ServiceProvider.GetRequiredService<VATPRCContext>();

        var data = await VatsimService.GetOnlineData(ct);

        await ClearFinalizedFlights(db, data, ct);
        foreach (var pilot in data.Pilots)
        {
            await UpdateFlight(db, pilot, ct);
        }
    }

    protected async ValueTask ClearFinalizedFlights(VATPRCContext db, VatsimData.VatsimData data, CancellationToken ct = default)
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

    protected async ValueTask UpdateFlight(VATPRCContext db, VatsimData.Pilot pilot, CancellationToken ct = default)
    {
        if (pilot.FlightPlan == null)
        {
            Logger.LogDebug("Ignore {Callsign} with no flight plan.", pilot.Callsign);
            return;
        }
        bool isOverflyChina = false;
        if (!IsChinaAirport(pilot.FlightPlan.Departure) && !IsChinaAirport(pilot.FlightPlan.Arrival))
        {
            // try
            // {
            //     var normalizedRoute = await FlightRoute.NormalizeRoute(
            //         db,
            //         pilot.FlightPlan.Departure,
            //         pilot.FlightPlan.Arrival,
            //         pilot.FlightPlan.Route);
            //     isOverflyChina = await db.PreferredRoute
            //         .Where(r => r.Arrival.Length > 4
            //             && r.Departure.Length > 4
            //             && normalizedRoute.Contains(r.Departure)
            //             && normalizedRoute.Contains(r.Arrival))
            //         .AnyAsync(ct);
            // }
            // catch (Exception) { }
            if (!isOverflyChina)
            {
                Logger.LogDebug("Ignore {Callsign} ({Departure}-{Arrival}) with no VATPRC airport.",
                    pilot.Callsign, pilot.FlightPlan.Departure, pilot.FlightPlan.Arrival);
                return;
            }
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
        flight.RawRoute = pilot.FlightPlan.Route;
        var aircraft = FlightPlan.ParseIcaoAircraftCode(pilot.FlightPlan.Aircraft, pilot.FlightPlan.Remarks);
        flight.Aircraft = aircraft.AircraftCode;
        flight.Equipment = aircraft.Equipment;
        flight.Transponder = aircraft.Transponder;
        flight.NavigationPerformance = aircraft.NavigationPerformance;

        await db.SaveChangesAsync(ct);
        Logger.LogDebug("Saved flight to database: {Callsign}", pilot.Callsign);
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
