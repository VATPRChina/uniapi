using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class RestrictedAirwayValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(FlightLeg leg, int index, INavdataProvider navdata, AirwayFix? fromLeg, AirwayFix? toLeg)
    {
        if (fromLeg == null || toLeg == null) yield break;

        if (fromLeg.FixIcaoCode.StartsWith("Z")
            && toLeg.FixIcaoCode.StartsWith("Z")
            && (fromLeg.AirwayIdentifier!.StartsWith("V") || fromLeg.AirwayIdentifier!.StartsWith("X")))
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
