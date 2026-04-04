using System.ComponentModel;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators.LegValidators;

public class DirectValidator : ILegValidator
{
    public bool RunOnMatchedRoute => false;

    public async IAsyncEnumerable<ValidationMessage> Validate(FlightLeg leg, int index, INavdataProvider navdata, AirwayFix? fromLeg, AirwayFix? toLeg)
    {
        if (leg.LegId == null && leg.LegIdentifier == "DCT"
            && leg.From.Type != FlightFix.FixType.Airport
            && leg.To.Type != FlightFix.FixType.Airport)
        {
            var fromFix = await navdata.GetFullQualifiedFixIdentifier(leg.From.Id, leg.From.Type switch
            {
                FlightFix.FixType.Airport => INavdataProvider.FixType.Unknown,
                FlightFix.FixType.Waypoint => INavdataProvider.FixType.Waypoint,
                FlightFix.FixType.Vhf => INavdataProvider.FixType.Vhf,
                FlightFix.FixType.Ndb => INavdataProvider.FixType.Ndb,
                FlightFix.FixType.GeoCoord => INavdataProvider.FixType.Unknown,
                FlightFix.FixType.Unknown => INavdataProvider.FixType.Unknown,
                _ => throw new InvalidEnumArgumentException($"Unexpected fix type: {leg.From.Type}"),
            });
            var toFix = await navdata.GetFullQualifiedFixIdentifier(leg.To.Id, leg.To.Type switch
            {
                FlightFix.FixType.Airport => INavdataProvider.FixType.Unknown,
                FlightFix.FixType.Waypoint => INavdataProvider.FixType.Waypoint,
                FlightFix.FixType.Vhf => INavdataProvider.FixType.Vhf,
                FlightFix.FixType.Ndb => INavdataProvider.FixType.Ndb,
                FlightFix.FixType.GeoCoord => INavdataProvider.FixType.Unknown,
                FlightFix.FixType.Unknown => INavdataProvider.FixType.Unknown,
                _ => throw new InvalidEnumArgumentException($"Unexpected fix type: {leg.To.Type}"),
            });
            if ((fromFix == null || fromFix.StartsWith("Z"))
                && (toFix == null || toFix.StartsWith("Z")))
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
}
