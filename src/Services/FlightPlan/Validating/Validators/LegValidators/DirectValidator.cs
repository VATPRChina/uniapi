using System.ComponentModel;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Fixes;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class DirectValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(Leg leg, int index, INavdataProvider navdata)
    {
        if (leg is not DirectLeg directLeg || directLeg.From is Airport || directLeg.To is Airport)
        {
            yield break;
        }

        // TODO: validate geo fix only within China airspace
        if ((leg.From is not FixWithIdentifier fromFix || fromFix.Identifier.StartsWith("Z"))
            && (leg.To is not FixWithIdentifier toFix || toFix.Identifier.StartsWith("Z")))
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                FieldParam = index,
                Type = ValidationMessage.ViolationType.Direct,
            };
        }
    }
}
