using System.ComponentModel;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public partial class FlightController(
    Database DbContext,
    ILogger<FlightController> Logger,
    RouteParseService RouteParse,
    IUserAccessor userAccessor) : ControllerBase
{
    [HttpGet("active")]
    [AllowAnonymous]
    public async Task<IEnumerable<FlightDto>> GetActive()
    {
        return await DbContext.Flight
            .Where(f => f.FinalizedAt == null)
            .OrderBy(f => f.Callsign)
            .Select(f => FlightDto.From(f))
            .ToListAsync();
    }

    [HttpGet("by-callsign/{callsign}")]
    [AllowAnonymous]
    public async Task<FlightDto> GetByCallsign(string callsign)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        return FlightDto.From(flight);
    }

    [HttpGet("by-callsign/{callsign}/warnings")]
    [AllowAnonymous]
    public async Task<IEnumerable<WarningMessage>> GetWarningByCallsign(string callsign, CancellationToken ct)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        var parsedRoute = await RouteParse.ParseRouteAsync(flight.RawRoute, flight.Departure, flight.Arrival, ct);
        var violations = await RouteParse.ValidateFlight(flight, parsedRoute, ct);

        return violations.Select(WarningMessage.From);
    }

    public record FlightLeg
    {
        public required FlightFix From { get; set; }
        public required FlightFix To { get; set; }
        public required string LegIdentifier { get; set; }
    }

    public record FlightFix
    {
        public required string Identifier { get; set; }
    }

    [HttpGet("by-callsign/{callsign}/route")]
    [AllowAnonymous]
    public async Task<IList<FlightLeg>> GetRouteByCallsign(string callsign, CancellationToken ct)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        try
        {
            var parsedRoute = await RouteParse.ParseRouteAsync(flight.RawRoute, flight.Departure, flight.Arrival, ct);
            return parsedRoute.Select(leg => new FlightLeg
            {
                From = new FlightFix
                {
                    Identifier = leg.From.Identifier,
                },
                To = new FlightFix
                {
                    Identifier = leg.To.Identifier,
                },
                LegIdentifier = leg.LegIdentifier,
            }).ToList();
        }
        catch (Exception e)
        {
            Logger.LogError(e, "Failed to parse route for flight {Callsign}", flight.Callsign);
        }
        return null!;
    }

    public record TemporaryFlightQuery
    {
        [FromQuery]
        [ModelBinder(Name = "departure")]
        public required string Departure { get; init; }

        [FromQuery]
        [ModelBinder(Name = "arrival")]
        public required string Arrival { get; init; }

        [FromQuery]
        [ModelBinder(Name = "aircraft")]
        public string Aircraft { get; init; } = string.Empty;

        [FromQuery]
        [ModelBinder(Name = "equipment")]
        public string Equipment { get; init; } = string.Empty;

        [FromQuery]
        [ModelBinder(Name = "navigation_performance")]
        public string NavigationPerformance { get; init; } = string.Empty;

        [FromQuery]
        [ModelBinder(Name = "transponder")]
        public string Transponder { get; init; } = string.Empty;

        [FromQuery]
        [ModelBinder(Name = "raw_route")]
        public string RawRoute { get; init; } = string.Empty;

        [FromQuery]
        [ModelBinder(Name = "cruising_level")]
        public long CruisingLevel { get; init; } = 0;
    }

    [HttpGet("temporary/by-plan/warnings")]
    [Authorize(Roles = Models.UserRoles.ApiClient)]
    public async Task<IEnumerable<WarningMessage>> GetByFlightPlanId(TemporaryFlightQuery query, CancellationToken ct = default)
    {
        var flight = new Flight
        {
            Id = Ulid.Empty,
            Cid = string.Empty,
            Callsign = string.Empty,
            LastObservedAt = DateTimeOffset.UtcNow,
            Departure = query.Departure,
            Arrival = query.Arrival,
            Equipment = query.Equipment,
            NavigationPerformance = query.NavigationPerformance,
            Transponder = query.Transponder,
            RawRoute = query.RawRoute,
            Aircraft = query.Aircraft,
            Altitude = 0,
            CruisingLevel = query.CruisingLevel,
        };

        var parsedRoute = await RouteParse.ParseRouteAsync(flight.RawRoute, flight.Departure, flight.Arrival, ct);
        var violations = await RouteParse.ValidateFlight(flight, parsedRoute, ct);

        return violations.Select(WarningMessage.From);
    }

    [HttpGet("mine")]
    public async Task<FlightDto> GetMyFlight()
    {
        var user = await userAccessor.GetUser();

        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Cid == user.Cid && f.FinalizedAt == null)
            ?? throw new ApiError.FlightNotFoundForCid(user.Cid);
        return FlightDto.From(flight);
    }
}
