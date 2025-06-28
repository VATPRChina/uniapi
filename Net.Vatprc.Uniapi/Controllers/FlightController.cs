using System.ComponentModel;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public class FlightController(VATPRCContext DbContext, ILogger<FlightController> Logger, RouteParseService RouteParse) : ControllerBase
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

    public enum WarningMessageCode
    {
        [Description("The aircraft does not support RVSM.")]
        no_rvsm,

        [Description("The aircraft does not support RNAV1.")]
        no_rnav1,

        [Description("The aircraft does not support RNAV1.")]
        no_rnav1_equipment,

        [Description("The aircraft does not support RNAV1.")]
        no_rnav1_pbn,

        [Description("The aircraft supports RNP AR with RF.")]
        rnp_ar,

        [Description("The aircraft supports RNP AR without RF.")]
        rnp_ar_without_rf,

        [Description("The aircraft does not have a transponder.")]
        no_transponder,

        [Description("There is no preferred route designated by CAAC.")]
        no_preferred_route,

        [Description("The flight does not follow the preferred route designated by CAAC. The parameter is the preferred route.")]
        not_preferred_route,

        [Description("The route cannot be parsed with the navdata on the server.")]
        parse_route_failed,
    }

    public record WarningMessage
    {
        public required WarningMessageCode MessageCode { get; init; }
        public string? Parameter { get; init; } = null;
    }

    [HttpGet("by-callsign/{callsign}/warnings")]
    [AllowAnonymous]
    public async Task<IEnumerable<WarningMessage>> GetWarningByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        var result = new List<WarningMessage>();
        if (!flight.SupportRvsm)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.no_rvsm });
        }
        if (!flight.SupportRnav1)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.no_rnav1 });
        }
        if (!flight.SupportRnav1Equipment)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.no_rnav1_equipment });
        }
        if (!flight.SupportRnav1Pbn)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.no_rnav1_pbn });
        }
        if (flight.SupportRnpArWithRf)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.rnp_ar });
        }
        if (flight.SupportRnpArWithoutRf && !flight.SupportRnpArWithRf)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.rnp_ar_without_rf });
        }
        if (!flight.HasTransponder)
        {
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.no_transponder });
        }
        var simplifiedRoute = FlightRoute.SimplifyRoute(flight.RawRoute);

        string normalizedRoute;
        try { normalizedRoute = await FlightRoute.NormalizeRoute(DbContext, flight.Departure, flight.Arrival, flight.RawRoute); }
        catch (Exception e)
        {
            normalizedRoute = simplifiedRoute;
            result.Add(new WarningMessage { MessageCode = WarningMessageCode.parse_route_failed, Parameter = e.Message });
        }
        return result;
    }

    [HttpGet("by-callsign/{callsign}/__route")]
    [AllowAnonymous]
    public async Task<IList<RouteToken>> GetRouteByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        try
        {
            var parsedRoute = await RouteParse.ParseRouteAsync(flight.RawRoute, flight.Departure, flight.Arrival);
            return parsedRoute;
        }
        catch (Exception e)
        {
            Logger.LogError(e, "Failed to parse route for flight {Callsign}", flight.Callsign);
            SentrySdk.CaptureException(e, scope =>
            {
                scope.SetExtra("Callsign", flight.Callsign);
                scope.SetExtra("RawRoute", flight.RawRoute);
            });
        }
        return null!;
    }
}
