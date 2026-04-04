using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PlanValidators;

public class RvsmValidator : IPlanValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan)
    {
        if (!plan.SupportRvsm)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Equipment,
                Type = ValidationMessage.ViolationType.NoRvsm,
            };
        }
    }
}
