using Net.Vatprc.Uniapi.External.FlightPlan.RouteParser;
using Net.Vatprc.Uniapi.Models.Acdm;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Validator;

public class Validator(Flight flight, IList<FlightLeg> legs, INavdataProvider navdata)
{
    protected readonly Flight Flight = flight;
    protected readonly IList<FlightLeg> Legs = legs;
    protected readonly INavdataProvider Navdata = navdata;

    protected readonly IList<Violation> Violations = [];

    public async Task<IList<Violation>> Validate()
    {
        if (!Flight.SupportRvsm)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                FieldParam = string.Empty,
                Type = Violation.ViolationType.NoRvsm,
            });
        }

        if (!Flight.SupportRnav1Equipment)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                FieldParam = string.Empty,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (!Flight.SupportRnav1Pbn)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                FieldParam = string.Empty,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (Flight.SupportRnpArWithoutRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                FieldParam = string.Empty,
                Type = Violation.ViolationType.RnpArWithoutRf,
            });
        }

        if (Flight.SupportRnpArWithRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                FieldParam = string.Empty,
                Type = Violation.ViolationType.RnpAr,
            });
        }

        foreach (var (leg, index) in Legs.Select((l, i) => (l, i)))
        {
            if (leg.LegId == null && leg.LegIdentifier == "DCT"
                && leg.From.Type != FlightFix.FixType.Airport
                && leg.To.Type != FlightFix.FixType.Airport)
            {
                Violations.Add(new Violation
                {
                    Field = Violation.FieldType.Route,
                    FieldParam = index.ToString(),
                    Type = Violation.ViolationType.Direct,
                });
            }

            if (leg.LegId != null)
            {
                var (fromLegId, toLegId) = leg.LegId.Value;
                var fromLeg = await Navdata.GetAirwayFix(fromLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {fromLegId}");
                var toLeg = await Navdata.GetAirwayFix(toLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {toLegId}");
                if (fromLeg.DirectionalRestriction == 'F' && fromLeg.SequenceNumber >= toLeg.SequenceNumber)
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index.ToString(),
                        Type = Violation.ViolationType.Direct,
                    });
                }
                if (fromLeg.DirectionalRestriction == 'B' && fromLeg.SequenceNumber <= toLeg.SequenceNumber)
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index.ToString(),
                        Type = Violation.ViolationType.Direct,
                    });
                }
                if (toLeg.DirectionalRestriction == 'F' && toLeg.SequenceNumber >= fromLeg.SequenceNumber)
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index.ToString(),
                        Type = Violation.ViolationType.Direct,
                    });
                }
                if (toLeg.DirectionalRestriction == 'F' && toLeg.SequenceNumber >= fromLeg.SequenceNumber)
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index.ToString(),
                        Type = Violation.ViolationType.Direct,
                    });
                }
            }
        }

        return Violations;
    }
}
