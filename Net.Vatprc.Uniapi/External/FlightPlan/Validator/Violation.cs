namespace Net.Vatprc.Uniapi.External.FlightPlan.Validator;

public class Violation
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
    }

    public enum ViolationType
    {
        NoRvsm,
        NoRnav1,
        RnpAr,
        RnpArWithoutRf,
        NoTransponder,
        Direct,
        LegDirection,
        AirwayRequireApproval,
        NotRecommendedRoute,
        CruisingLevelMismatch,
        CruisingLevelTooLow,
        CruisingLevelNotAllowed,
    }
}
