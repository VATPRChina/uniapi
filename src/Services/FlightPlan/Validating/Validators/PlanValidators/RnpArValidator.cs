using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PlanValidators;

public class RnpArValidator : IPlanValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan)
    {
        if (plan.SupportRnpArWithoutRf)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.RnpArWithoutRf,
            };
        }

        if (plan.SupportRnpArWithRf)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.RnpAr,
            };
        }
    }
}
