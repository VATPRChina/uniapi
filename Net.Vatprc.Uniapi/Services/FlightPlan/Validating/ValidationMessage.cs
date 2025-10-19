namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating;

public class ValidationMessage
{
    public required FieldType Field { get; set; }
    public int? FieldParam { get; set; }
    public required ViolationType Type { get; set; }
    public string? Param { get; set; }

    public enum FieldType
    {
        Equipment,
        Transponder,
        NavigationPerformance,
        Route,
        CruisingLevel,
    }

    public enum ViolationType
    {
        NoRvsm,
        NoRnav1,
        RnpAr,
        RnpArWithoutRf,
        NoTransponder,
        Direct,
        LegDirectionViolation,
        AirwayRequireApproval,
        NotRecommendedRoute,
        CruisingLevelMismatch,
        CruisingLevelTooLow,
        CruisingLevelNotAllowed,
        RouteMatchPreferred,
    }
}
