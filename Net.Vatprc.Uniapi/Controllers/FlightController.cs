using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public class FlightController(VATPRCContext DbContext, ILogger<FlightController> Logger) : ControllerBase
{
    public record FlightDto
    {
        public Ulid Id { get; init; }
        public string Cid { get; init; }
        public string Callsign { get; init; }
        public DateTimeOffset LastObservedAt { get; init; }
        public string Departure { get; init; }
        public string Arrival { get; init; }
        public string Equipment { get; init; }
        public string NavigationPerformance { get; init; }
        public string Transponder { get; init; }
        public string RawRoute { get; init; }
        public string __SimplifiedRoute { get; init; }
        public string Aircraft { get; init; }
        public string? __NormalizedRoute { get; init; }

        public FlightDto(Flight flight, string? normalizedRoute = null)
        {
            Id = flight.Id;
            Cid = flight.Cid;
            Callsign = flight.Callsign;
            LastObservedAt = flight.LastObservedAt;
            Departure = flight.Departure;
            Arrival = flight.Arrival;
            Equipment = flight.Equipment;
            NavigationPerformance = flight.NavigationPerformance;
            Transponder = flight.Transponder;
            RawRoute = flight.RawRoute;
            __SimplifiedRoute = FlightRoute.SimplifyRoute(flight.RawRoute);
            Aircraft = flight.Aircraft;
            __NormalizedRoute = normalizedRoute;
        }
    }

    [HttpGet("active")]
    [AllowAnonymous]
    public async Task<IEnumerable<FlightDto>> GetActive()
    {
        return await DbContext.Flight
            .Where(f => f.FinalizedAt == null)
            .OrderBy(f => f.Callsign)
            .Select(f => new FlightDto(f, null))
            .ToListAsync();
    }

    [HttpGet("by-callsign/{callsign}")]
    [AllowAnonymous]
    public async Task<FlightDto> GetByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        return new FlightDto(flight, await FlightRoute.TryNormalizeRoute(DbContext, flight.Departure, flight.Arrival, flight.RawRoute));
    }

    public record WarningMessage
    {
        public required string MessageCode { get; init; }
        public string? Parameter { get; init; } = null;
    }

    [EndpointDescription("""
        List of warnings:

        - `no_rvsm`: The aircraft does not support RVSM.
        - `no_rnav1`: The aircraft does not support RNAV1.
        - `rnp_ar`: The aircraft supports RNP AR with RF.
        - `rnp_ar_without_rf`: The aircraft supports RNP AR without RF.
        - `no_transponder`: The aircraft does not have a transponder.
        - `no_preferred_route`: There is no preferred route designated by CAAC.
        - `not_preferred_route`: The flight does not follow the preferred route designated by CAAC.
           The parameter is the preferred route.
        - `parse_route_failed`: The route cannot be parsed with the navdata on the server.
        """)]
    [HttpGet("by-callsign/{callsign}/warnings")]
    [AllowAnonymous]
    public async Task<IEnumerable<WarningMessage>> GetWarningByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        var result = new List<WarningMessage>();
        if (!flight.SupportRvsm)
        {
            result.Add(new WarningMessage { MessageCode = "no_rvsm" });
        }
        if (!flight.SupportRnav1)
        {
            result.Add(new WarningMessage { MessageCode = "no_rnav1" });
        }
        if (flight.SupportRnpArWithRf)
        {
            result.Add(new WarningMessage { MessageCode = "rnp_ar" });
        }
        if (flight.SupportRnpArWithoutRf && !flight.SupportRnpArWithRf)
        {
            result.Add(new WarningMessage { MessageCode = "rnp_ar_without_rf" });
        }
        if (!flight.HasTransponder)
        {
            result.Add(new WarningMessage { MessageCode = "no_transponder" });
        }
        var simplifiedRoute = FlightRoute.SimplifyRoute(flight.RawRoute);

        string normalizedRoute;
        try { normalizedRoute = await FlightRoute.NormalizeRoute(DbContext, flight.Departure, flight.Arrival, flight.RawRoute); }
        catch (Exception e)
        {
            normalizedRoute = simplifiedRoute;
            result.Add(new WarningMessage { MessageCode = "parse_route_failed", Parameter = e.Message });
        }

        var preferredRoutes = await DbContext.PreferredRoute
            .Where(pr => (pr.Departure == flight.Departure && pr.Arrival == flight.Arrival)
                || (pr.Departure == flight.Departure && normalizedRoute.Contains(pr.Arrival) && pr.Arrival.Length > 4)
                || (pr.Arrival == flight.Arrival && normalizedRoute.Contains(pr.Departure) && pr.Departure.Length > 4)
                || (pr.Arrival.Length > 4
                    && pr.Departure.Length > 4
                    && normalizedRoute.Contains(pr.Departure)
                    && normalizedRoute.Contains(pr.Arrival)))
            .ToListAsync();
        if (preferredRoutes.Count == 0)
        {
            result.Add(new WarningMessage { MessageCode = "no_preferred_route" });
        }
        foreach (var pr in preferredRoutes)
        {
            Logger.LogDebug("Found route from {Departure} to {Arrival} via {Route}", pr.Departure, pr.Arrival, pr.RawRoute);
        }
        if (preferredRoutes.Count > 0 && !preferredRoutes.Any(preferred => simplifiedRoute.Contains(preferred.RawRoute)))
        {
            result.Add(new WarningMessage { MessageCode = "not_preferred_route", Parameter = string.Join(";", preferredRoutes.Select(p => p.RawRoute)) });
        }
        return result;
    }
}
