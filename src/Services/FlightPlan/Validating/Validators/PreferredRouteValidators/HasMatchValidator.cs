using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PreferredRouteValidators;

public class HasMatchValidator : IPreferredRouteMatchValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan, PreferredRoute? prefRoute, IList<PreferredRoute> prefRoutes)
    {
        if (prefRoute != null)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                Type = ValidationMessage.ViolationType.RouteMatchPreferred,
                Param = (prefRoute.IsPublic ? prefRoute.RawRoute : plan.RawRoute) ?? string.Empty,
            };
        }

        if (prefRoute == null && prefRoutes.Any())
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                Type = ValidationMessage.ViolationType.NotRecommendedRoute,
                Param = string.Join(",", prefRoutes
                    .Where(r => !r.IsPublic)
                    .OrderBy(r => r.Id)
                    .Select(r => r.RawRoute)),
            };
        }
    }
}
