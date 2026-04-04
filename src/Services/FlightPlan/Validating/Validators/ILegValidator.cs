using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators;

public interface ILegValidator
{
    public bool RunOnMatchedRoute { get; }
    public IAsyncEnumerable<ValidationMessage> Validate(FlightLeg leg, int index, INavdataProvider navdata, AirwayFix? fromLeg, AirwayFix? toLeg);
}
