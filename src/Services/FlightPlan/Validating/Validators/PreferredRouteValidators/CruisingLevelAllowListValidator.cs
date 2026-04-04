using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PreferredRouteValidators;

public class CruisingLevelAllowListValidator : IPreferredRouteMatchValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan, PreferredRoute? prefRoute, IList<PreferredRoute> prefRoutes)
    {
        if (prefRoute == null) yield break;

        var cruisingLevelType = AltitudeHelper.GetLevelRestrictionTypeFromCruisingLevel((int)plan.CruisingLevel);
        if (!AltitudeHelper.IsFlightLevelTypeMatching(cruisingLevelType, prefRoute.CruisingLevelRestriction))
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.CruisingLevel,
                Type = ValidationMessage.ViolationType.CruisingLevelMismatch,
                Param = prefRoute.CruisingLevelRestriction switch
                {
                    PreferredRoute.LevelRestrictionType.StandardEven => "standard_even",
                    PreferredRoute.LevelRestrictionType.StandardOdd => "standard_odd",
                    PreferredRoute.LevelRestrictionType.Standard => "standard",
                    PreferredRoute.LevelRestrictionType.FlightLevelEven => "flight_level_even",
                    PreferredRoute.LevelRestrictionType.FlightLevelOdd => "flight_level_odd",
                    PreferredRoute.LevelRestrictionType.FlightLevel => "flight_level",
                    _ => "unknown"
                },
                DebugMessage = $"Cruising level type mismatch: {cruisingLevelType} vs {prefRoute.CruisingLevelRestriction}"
            };
        }
    }
}
