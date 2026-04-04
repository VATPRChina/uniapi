using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators;

public interface IPreferredRouteMatchValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan, PreferredRoute? prefRoute, IList<PreferredRoute> prefRoutes);
}
