using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PlanValidators;

public class Rnav1PbnValidator : IPlanValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan)
    {
        if (!plan.SupportRnav1Pbn)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.NoRnav1,
            };
        }
    }
}
