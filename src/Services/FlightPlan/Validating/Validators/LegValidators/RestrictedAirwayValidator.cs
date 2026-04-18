using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class RestrictedAirwayValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(Leg leg, int index, INavdataProvider navdata)
    {
        if (leg is not AirwayLeg airwayLeg) yield break;

        if (airwayLeg.From.Identifier.StartsWith('Z')
            && airwayLeg.To.Identifier.StartsWith('Z')
            && (airwayLeg.Identifier.StartsWith('V') || airwayLeg.Identifier.StartsWith('X')))
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                FieldParam = index,
                Type = ValidationMessage.ViolationType.AirwayRequireApproval,
            };
        }
    }
}
