using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class DirectionValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(Leg leg, int index, INavdataProvider navdata)
    {
        if (leg is not AirwayLeg airwayLeg)
        {
            yield break;
        }

        if (airwayLeg.Direction == AirwayLeg.AirwayDirection.BACKWARD)
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                FieldParam = index,
                Type = ValidationMessage.ViolationType.LegDirectionViolation,
            };
        }
    }
}
