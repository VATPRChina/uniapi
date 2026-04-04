using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class DirectionValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(FlightLeg leg, int index, INavdataProvider navdata, AirwayFix? fromLeg, AirwayFix? toLeg)
    {
        if (fromLeg == null || toLeg == null) yield break;

        if (fromLeg.SequenceNumber <= toLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'B')
        {
            yield return new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                FieldParam = index,
                Type = ValidationMessage.ViolationType.LegDirectionViolation,
            };
        }
        // TODO: test for KARSI[5720] * - TR[5750] F is bidirectional
        if (toLeg.SequenceNumber <= fromLeg.SequenceNumber && toLeg.DirectionalRestriction == 'F')
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
