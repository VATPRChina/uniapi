using System.ComponentModel;
using Net.Vatprc.Uniapi.Services.FlightPlan.Validating;

namespace Net.Vatprc.Uniapi.Dto;

public record WarningMessage
{

    public required WarningMessageCode MessageCode { get; set; }
    public required string? Parameter { get; set; }
    public required WarningMessageField Field { get; set; }
    public required int? FieldIndex { get; set; }

    public static WarningMessage From(ValidationMessage v) => new()
    {
        MessageCode = v.Type switch
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
        Field = v.Field switch
        {
            ValidationMessage.FieldType.Equipment => WarningMessageField.equipment,
            ValidationMessage.FieldType.Transponder => WarningMessageField.transponder,
            ValidationMessage.FieldType.NavigationPerformance => WarningMessageField.navigation_performance,
            ValidationMessage.FieldType.Route => WarningMessageField.route,
            ValidationMessage.FieldType.CruisingLevel => WarningMessageField.cruising_level,
            _ => throw new InvalidEnumArgumentException($"Unexpected violation field {v.Field}"),
        },
        Parameter = v.Param,
        FieldIndex = v.FieldParam,
    };
}
