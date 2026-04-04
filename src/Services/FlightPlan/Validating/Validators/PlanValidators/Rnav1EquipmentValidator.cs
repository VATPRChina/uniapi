using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.PlanValidators;

public class Rnav1EquipmentValidator : IPlanValidator
{
    public IEnumerable<ValidationMessage> Validate(Flight plan)
    {
        if (!plan.SupportRnav1Equipment && AltitudeHelper.IsInRvsm(plan.CruisingLevel))
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Equipment,
                Type = ValidationMessage.ViolationType.NoRnav1,
            };
        }
    }
}
