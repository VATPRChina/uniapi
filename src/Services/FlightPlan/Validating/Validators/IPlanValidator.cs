using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators;

public interface IPlanValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan);
}
