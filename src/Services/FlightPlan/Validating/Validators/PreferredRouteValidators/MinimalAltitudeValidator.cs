using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PreferredRouteValidators;

public class MinimalAltitudeValidator : IPreferredRouteMatchValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan, PreferredRoute? prefRoute, IList<PreferredRoute> prefRoutes)
    {
        if (prefRoute == null) yield break;

        if (plan.CruisingLevel < prefRoute.MinimalAltitude)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.CruisingLevel,
                Type = ValidationMessage.ViolationType.CruisingLevelTooLow,
                Param = prefRoute.MinimalAltitude.ToString()
            };
        }
    }
}
