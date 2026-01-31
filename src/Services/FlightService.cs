using System.Collections;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using Microsoft.Extensions.Caching.Memory;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services;

public class FlightService(ILogger<FlightService> logger, VatsimAdapter vatsimAdapter, ActivitySource activitySource)
{
    public async Task<IEnumerable<Flight>> GetFlightsAsync(CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity($"{nameof(FlightService)}.{nameof(GetFlightsAsync)}");
        var vatsimData = await vatsimAdapter.GetOnlineData(ct);
        var flights = vatsimData.Pilots
            .Select(TryMapVatsimFlight)
            .Where(flight => flight != null)
            .Select(flight => flight!)
            .ToList();
        return flights;
    }

    public async Task<Flight?> GetFlightByCallsignAsync(string callsign, CancellationToken ct = default)
    {
        var flights = await GetFlightsAsync(ct);
        return flights.FirstOrDefault(f => f.Callsign.Equals(callsign, StringComparison.OrdinalIgnoreCase));
    }

    public async Task<Flight?> GetFlightByCidAsync(string cid, CancellationToken ct = default)
    {
        var flights = await GetFlightsAsync(ct);
        return flights.FirstOrDefault(f => f.Cid.Equals(cid, StringComparison.OrdinalIgnoreCase));
    }

    protected Flight? TryMapVatsimFlight(Adapters.VatsimAdapterModels.Pilot pilot)
    {
        using var activity = activitySource.StartActivity($"{nameof(FlightService)}.{nameof(TryMapVatsimFlight)}");
        try
        {
            return MapVatsimFlight(pilot);
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Failed to map VATSIM pilot {Callsign} to Flight.", pilot.Callsign);
            return null;
        }
    }

    protected Flight? MapVatsimFlight(Adapters.VatsimAdapterModels.Pilot pilot)
    {
        if (pilot.FlightPlan == null)
        {
            logger.LogDebug("Ignore {Callsign} with no flight plan.", pilot.Callsign);
            return null;
        }
        bool isOverflyChina = false;
        if (!IsChinaAirport(pilot.FlightPlan.Departure) && !IsChinaAirport(pilot.FlightPlan.Arrival))
        {
            if (!isOverflyChina)
            {
                logger.LogDebug("Ignore {Callsign} ({Departure}-{Arrival}) with no VATPRC airport.",
                    pilot.Callsign, pilot.FlightPlan.Departure, pilot.FlightPlan.Arrival);
                return null;
            }
        }
        if (pilot.FlightPlan.Departure == pilot.FlightPlan.Arrival)
        {
            logger.LogDebug("Ignore {Callsign} with same departure and arrival airport.", pilot.Callsign);
            return null;
        }
        if (pilot.FlightPlan.FlightRules != "I")
        {
            logger.LogDebug("Ignore {Callsign} with flight rules {FlightRules} (not IFR).", pilot.Callsign, pilot.FlightPlan.FlightRules);
            return null;
        }
        logger.LogDebug("Discovered flight: {Callsign}", pilot.Callsign);

        var aircraft = FlightPlanUtils.ParseIcaoAircraftCode(pilot.FlightPlan.Aircraft, pilot.FlightPlan.Remarks);
        var flight = new Flight
        {
            Id = Ulid.NewUlid(),
            State = Flight.FlightState.UNKNOWN,
            Cid = pilot.Cid.ToString(),
            Callsign = pilot.Callsign,
            LastObservedAt = pilot.LastUpdated.ToUniversalTime(),
            Latitude = pilot.Latitude,
            Longitude = pilot.Longitude,
            Altitude = pilot.Altitude,
            Departure = pilot.FlightPlan.Departure,
            Arrival = pilot.FlightPlan.Arrival,
            CruiseTas = uint.Parse(pilot.FlightPlan.CruiseTas),
            CruisingLevel = ParseFlightAltitude(pilot.FlightPlan.Altitude),
            RawRoute = pilot.FlightPlan.Route,
            Aircraft = aircraft.AircraftCode,
            Equipment = aircraft.Equipment,
            Transponder = aircraft.Transponder,
            NavigationPerformance = aircraft.NavigationPerformance
        };

        logger.LogDebug("Saved flight to database: {Callsign}", pilot.Callsign);
        return flight;
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
}
