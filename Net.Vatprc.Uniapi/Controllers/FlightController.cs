using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public class FlightController(VATPRCContext DbContext) : ControllerBase
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

        public FlightDto(Flight flight)
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
        }
    }

    [HttpGet("active")]
    [AllowAnonymous]
    public async Task<IEnumerable<FlightDto>> GetActive()
    {
        return await DbContext.Flight
            .Where(f => f.FinalizedAt == null)
            .OrderBy(f => f.Callsign)
            .Select(f => new FlightDto(f))
            .ToListAsync();
    }

    [HttpGet("by-callsign/{callsign}")]
    [AllowAnonymous]
    public async Task<FlightDto> GetByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        return new FlightDto(flight);
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
        var preferredRoutes = await DbContext.PreferredRoute
            .Where(pr => pr.Departure == flight.Departure && pr.Arrival == flight.Arrival)
            .ToListAsync();
        if (preferredRoutes.Count == 0)
        {
            result.Add(new WarningMessage { MessageCode = "no_preferred_route" });
        }
        var simplifiedRoute = FlightRoute.SimplifyRoute(flight.RawRoute);
        if (preferredRoutes.Count > 0 && !preferredRoutes.Any(preferred => simplifiedRoute.Contains(preferred.RawRoute)))
        {
            result.Add(new WarningMessage { MessageCode = "not_preferred_route", Parameter = string.Join(";", preferredRoutes.Select(p => p.RawRoute)) });
        }
        // TODO: check flight passing P.R. China airspace border
        return result;
    }
}
