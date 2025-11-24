using System.ComponentModel;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Services.FlightPlan.Validating;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Flight information.
/// </summary>
[ApiController, Route("api/flights")]
public class FlightController(
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

    public enum WarningMessageCode
    {
        [Description("The aircraft does not support RVSM.")]
        no_rvsm,

        [Description("The aircraft does not support RNAV1.")]
        no_rnav1,

        [Description("The aircraft supports RNP AR with RF.")]
        rnp_ar,

        [Description("The aircraft supports RNP AR without RF.")]
        rnp_ar_without_rf,

        [Description("The aircraft does not have a transponder.")]
        no_transponder,

        [Description("The route contains a direct segment.")]
        route_direct_segment,

        [Description("The route contains a leg with an incorrect direction.")]
        route_leg_direction,

        [Description("The route contains a leg requiring controller approval.")]
        airway_require_approval,

        [Description("The route is not recommended.")]
        not_preferred_route,

        [Description("The cruising level type does not match the preferred route.")]
        cruising_level_mismatch,

        [Description("The cruising level is too low for the preferred route.")]
        cruising_level_too_low,

        [Description("The cruising level is not allowed for the preferred route.")]
        cruising_level_not_allowed,

        [Description("The planned route is matching a preferred route.")]
        route_match_preferred,
    }

    public enum WarningMessageField
    {
        [Description("Equipment")]
        equipment,

        [Description("Transponder")]
        transponder,

        [Description("Navigation Performance")]
        navigation_performance,

        [Description("Route, with field index if applicable")]
        route,

        [Description("Cruising Level")]
        cruising_level,
    }

    public record WarningMessage(
        WarningMessageCode MessageCode,
        string? Parameter,
        WarningMessageField Field,
        int? FieldIndex)
    {
        public WarningMessage(ValidationMessage v) : this(
            MessageCode: v.Type switch
            {
                ValidationMessage.ViolationType.NoRvsm => WarningMessageCode.no_rvsm,
                ValidationMessage.ViolationType.NoRnav1 => WarningMessageCode.no_rnav1,
                ValidationMessage.ViolationType.RnpAr => WarningMessageCode.rnp_ar,
                ValidationMessage.ViolationType.RnpArWithoutRf => WarningMessageCode.rnp_ar_without_rf,
                ValidationMessage.ViolationType.NoTransponder => WarningMessageCode.no_transponder,
                ValidationMessage.ViolationType.Direct => WarningMessageCode.route_direct_segment,
                ValidationMessage.ViolationType.LegDirectionViolation => WarningMessageCode.route_leg_direction,
                ValidationMessage.ViolationType.AirwayRequireApproval => WarningMessageCode.airway_require_approval,
                ValidationMessage.ViolationType.NotRecommendedRoute => WarningMessageCode.not_preferred_route,
                ValidationMessage.ViolationType.CruisingLevelMismatch => WarningMessageCode.cruising_level_mismatch,
                ValidationMessage.ViolationType.CruisingLevelTooLow => WarningMessageCode.cruising_level_too_low,
                ValidationMessage.ViolationType.CruisingLevelNotAllowed => WarningMessageCode.cruising_level_not_allowed,
                ValidationMessage.ViolationType.RouteMatchPreferred => WarningMessageCode.route_match_preferred,
                _ => throw new InvalidEnumArgumentException($"Unexpected violation type {v.Type}"),
            },
            Field: v.Field switch
            {
                ValidationMessage.FieldType.Equipment => WarningMessageField.equipment,
                ValidationMessage.FieldType.Transponder => WarningMessageField.transponder,
                ValidationMessage.FieldType.NavigationPerformance => WarningMessageField.navigation_performance,
                ValidationMessage.FieldType.Route => WarningMessageField.route,
                ValidationMessage.FieldType.CruisingLevel => WarningMessageField.cruising_level,
                _ => throw new InvalidEnumArgumentException($"Unexpected violation field {v.Field}"),
            },
            Parameter: v.Param,
            FieldIndex: v.FieldParam)
        { }
    }

    [HttpGet("by-callsign/{callsign}/warnings")]
    [AllowAnonymous]
    public async Task<IEnumerable<WarningMessage>> GetWarningByCallsign(string callsign, CancellationToken ct)
    {
        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Callsign == callsign && f.FinalizedAt == null)
            ?? throw new ApiError.CallsignNotFound(callsign);
        var parsedRoute = await RouteParse.ParseRouteAsync(flight.RawRoute, flight.Departure, flight.Arrival, ct);
        var violations = await RouteParse.ValidateFlight(flight, parsedRoute, ct);

        return violations.Select(v => new WarningMessage(v));
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

        return violations.Select(v => new WarningMessage(v));
    }

    [HttpGet("mine")]
    public async Task<FlightDto> GetMyFlight()
    {
        var user = await userAccessor.GetUser();

        var flight = await DbContext.Flight.FirstOrDefaultAsync(f => f.Cid == user.Cid && f.FinalizedAt == null)
            ?? throw new ApiError.FlightNotFoundForCid(user.Cid);
        return new FlightDto(flight);
    }
}
