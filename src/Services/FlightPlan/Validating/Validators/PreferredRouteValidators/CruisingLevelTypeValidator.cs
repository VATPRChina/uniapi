using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PreferredRouteValidators;

public class CruisingLevelTypeValidator : IPreferredRouteMatchValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan, PreferredRoute? prefRoute, IList<PreferredRoute> prefRoutes)
    {
        if (prefRoute == null) yield break;

        if (prefRoute.AllowedAltitudes.Any() && !prefRoute.AllowedAltitudes.Contains((int)plan.CruisingLevel))
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.CruisingLevel,
                Type = ValidationMessage.ViolationType.CruisingLevelNotAllowed,
                Param = string.Join(",", prefRoute.AllowedAltitudes)
            };
        }
    }
}
