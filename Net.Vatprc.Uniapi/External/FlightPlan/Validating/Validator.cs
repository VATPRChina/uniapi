using Net.Vatprc.Uniapi.External.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Utils;
using Serilog;

namespace Net.Vatprc.Uniapi.External.FlightPlan.Validating;

public class Validator(Flight flight, IList<FlightLeg> legs, INavdataProvider navdata)
{
    protected readonly Flight Flight = flight;
    protected readonly IList<FlightLeg> Legs = legs;
    protected readonly INavdataProvider Navdata = navdata;

    protected readonly IList<Violation> Violations = [];

    protected static readonly Serilog.ILogger Logger = Log.ForContext<Validator>();

    public async Task<IList<Violation>> Validate()
    {
        if (!Flight.SupportRvsm)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                Type = Violation.ViolationType.NoRvsm,
            });
        }

        if (!Flight.SupportRnav1Equipment)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Equipment,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (!Flight.SupportRnav1Pbn)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.NoRnav1,
            });
        }

        if (Flight.SupportRnpArWithoutRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.RnpArWithoutRf,
            });
        }

        if (Flight.SupportRnpArWithRf)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.NavigationPerformance,
                Type = Violation.ViolationType.RnpAr,
            });
        }

        var prefRoutes = await Navdata.GetRecommendedRoutes(Flight.Departure, Flight.Arrival);
        Logger.Information("Recommended routes for {Dep} to {Arr}: {Routes}",
            Flight.Departure, Flight.Arrival, prefRoutes);
        bool foundMatchingRoute = prefRoutes.Count == 0;
        foreach (var prefRte in prefRoutes)
        {
            var prefRteParsed = await new RouteParser(prefRte.RawRoute, Navdata).Parse();
            if (EnrouteRouteComparator.IsRouteMatchingExpected(Legs, prefRteParsed))
            {
                foundMatchingRoute = true;
                Logger.Information("Found matching route: {Route}", prefRte);

                var cruisingLevelType = AltitudeHelper.GetLevelRestrictionTypeFromCruisingLevel((int)Flight.CruisingLevel);
                if (!AltitudeHelper.IsFlightLevelTypeMatching(cruisingLevelType, prefRte.CruisingLevelRestriction))
                {
                    Logger.Information("Cruising level type mismatch: {Expected} vs {Actual}",
                        prefRte.CruisingLevelRestriction, cruisingLevelType);
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.CruisingLevel,
                        Type = Violation.ViolationType.CruisingLevelMismatch,
                        Param = prefRte.CruisingLevelRestriction switch
                        {
                            Models.Navdata.PreferredRoute.LevelRestrictionType.StandardEven => "standard_even",
                            Models.Navdata.PreferredRoute.LevelRestrictionType.StandardOdd => "standard_odd",
                            Models.Navdata.PreferredRoute.LevelRestrictionType.Standard => "standard",
                            Models.Navdata.PreferredRoute.LevelRestrictionType.FlightLevelEven => "flight_level_even",
                            Models.Navdata.PreferredRoute.LevelRestrictionType.FlightLevelOdd => "flight_level_odd",
                            Models.Navdata.PreferredRoute.LevelRestrictionType.FlightLevel => "flight_level",
                            _ => "unknown"
                        }
                    });
                }

                if (prefRte.AllowedAltitudes.Any() && !prefRte.AllowedAltitudes.Contains((int)Flight.CruisingLevel))
                {
                    Logger.Information("Cruising level {CruisingLevel} is not allowed by preferred route {Route}",
                        Flight.CruisingLevel, prefRte);
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.CruisingLevel,
                        Type = Violation.ViolationType.CruisingLevelNotAllowed,
                        Param = string.Join(",", prefRte.AllowedAltitudes)
                    });
                }

                if (Flight.CruisingLevel < prefRte.MinimalAltitude)
                {
                    Logger.Information("Cruising level {CruisingLevel} is below minimal altitude {MinimalAltitude} for preferred route {Route}",
                        Flight.CruisingLevel, prefRte.MinimalAltitude, prefRte);
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.CruisingLevel,
                        Type = Violation.ViolationType.CruisingLevelTooLow,
                        Param = prefRte.MinimalAltitude.ToString()
                    });
                }

                break;
            }
            else
            {
                Logger.Information("Expected route {Expected} does not match.", prefRte);
            }
        }
        if (!foundMatchingRoute)
        {
            Violations.Add(new Violation
            {
                Field = Violation.FieldType.Route,
                Type = Violation.ViolationType.NotRecommendedRoute,
                Param = string.Join(",", prefRoutes.Select(r => r.RawRoute)),
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
                    FieldParam = index,
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

                Logger.Information("Validating airway leg {FromLeg} to {ToLeg} for flight {FlightId}",
                    fromLeg.FixIdentifier, toLeg.FixIdentifier, Flight.Id);
                Logger.Information("Restrictions: From leg: {FromLeg}, To leg: {ToLeg}", fromLeg.DirectionalRestriction, toLeg.DirectionalRestriction);
                Logger.Information("Sequence numbers: From leg: {FromSeq}, To leg: {ToSeq}",
                    fromLeg.SequenceNumber, toLeg.SequenceNumber);
                if (fromLeg.SequenceNumber <= toLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'B')
                {
                    Logger.Information("Violation found: From leg is backward.");
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.LegDirection,
                    });
                }
                if (toLeg.SequenceNumber <= fromLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'F')
                {
                    Logger.Information("Violation found: From leg is forward.");
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.LegDirection,
                    });
                }

                if (fromLeg.FixIcaoCode.StartsWith("Z")
                    && (fromLeg.AirwayIdentifier!.StartsWith("V") || fromLeg.AirwayIdentifier!.StartsWith("X")))
                {
                    Violations.Add(new Violation
                    {
                        Field = Violation.FieldType.Route,
                        FieldParam = index,
                        Type = Violation.ViolationType.AirwayRequireApproval,
                    });
                }
            }
        }

        return Violations;
    }

}
