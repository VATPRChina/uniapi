using System.ComponentModel;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating;

public class Validator(
    Flight flight,
    IList<FlightLeg> legs,
    INavdataProvider navdata,
    RouteParserFactory routeParserFactory,
    ILogger<Validator> Logger,
    ILoggerFactory loggerFactory)
{
    protected readonly Flight Flight = flight;
    protected readonly IList<FlightLeg> Legs = legs;
    protected readonly INavdataProvider Navdata = navdata;

    protected readonly IList<ValidationMessage> Messages = [];

    protected readonly RouteParserFactory RouteParserFactory = routeParserFactory;

    public async Task<IList<ValidationMessage>> Validate(CancellationToken ct = default)
    {
        if (!Flight.SupportRvsm)
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Equipment,
                Type = ValidationMessage.ViolationType.NoRvsm,
            });
        }

        if (!Flight.SupportRnav1Equipment && AltitudeHelper.IsInRvsm(Flight.CruisingLevel))
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Equipment,
                Type = ValidationMessage.ViolationType.NoRnav1,
            });
        }

        if (!Flight.SupportRnav1Pbn)
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.NoRnav1,
            });
        }

        if (Flight.SupportRnpArWithoutRf)
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.RnpArWithoutRf,
            });
        }

        if (Flight.SupportRnpArWithRf)
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.NavigationPerformance,
                Type = ValidationMessage.ViolationType.RnpAr,
            });
        }

        var prefRoutes = await Navdata.GetRecommendedRoutes(Flight.Departure, Flight.Arrival);
        Logger.LogInformation("Recommended routes for {Dep} to {Arr}: {Routes}",
            Flight.Departure, Flight.Arrival, prefRoutes);
        PreferredRoute? matchingRoute = null;
        foreach (var prefRte in prefRoutes)
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Validation cancelled.");
                return Messages;
            }
            var prefRteParsed = await RouteParserFactory.Create(prefRte.RawRoute, Navdata).Parse(ct);
            if (EnrouteRouteComparator.IsRouteMatchingExpected(Legs, prefRteParsed, loggerFactory.CreateLogger<EnrouteRouteComparator>(), ct))
            {
                matchingRoute = prefRte;
                Logger.LogInformation("Found matching route: {Route}", prefRte);

                var cruisingLevelType = AltitudeHelper.GetLevelRestrictionTypeFromCruisingLevel((int)Flight.CruisingLevel);
                if (!AltitudeHelper.IsFlightLevelTypeMatching(cruisingLevelType, prefRte.CruisingLevelRestriction))
                {
                    Logger.LogInformation("Cruising level type mismatch: {Expected} vs {Actual}",
                        prefRte.CruisingLevelRestriction, cruisingLevelType);
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.CruisingLevel,
                        Type = ValidationMessage.ViolationType.CruisingLevelMismatch,
                        Param = prefRte.CruisingLevelRestriction switch
                        {
                            PreferredRoute.LevelRestrictionType.StandardEven => "standard_even",
                            PreferredRoute.LevelRestrictionType.StandardOdd => "standard_odd",
                            PreferredRoute.LevelRestrictionType.Standard => "standard",
                            PreferredRoute.LevelRestrictionType.FlightLevelEven => "flight_level_even",
                            PreferredRoute.LevelRestrictionType.FlightLevelOdd => "flight_level_odd",
                            PreferredRoute.LevelRestrictionType.FlightLevel => "flight_level",
                            _ => "unknown"
                        }
                    });
                }

                if (prefRte.AllowedAltitudes.Any() && !prefRte.AllowedAltitudes.Contains((int)Flight.CruisingLevel))
                {
                    Logger.LogInformation("Cruising level {CruisingLevel} is not allowed by preferred route {Route}",
                        Flight.CruisingLevel, prefRte);
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.CruisingLevel,
                        Type = ValidationMessage.ViolationType.CruisingLevelNotAllowed,
                        Param = string.Join(",", prefRte.AllowedAltitudes)
                    });
                }

                if (Flight.CruisingLevel < prefRte.MinimalAltitude)
                {
                    Logger.LogInformation("Cruising level {CruisingLevel} is below minimal altitude {MinimalAltitude} for preferred route {Route}",
                        Flight.CruisingLevel, prefRte.MinimalAltitude, prefRte);
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.CruisingLevel,
                        Type = ValidationMessage.ViolationType.CruisingLevelTooLow,
                        Param = prefRte.MinimalAltitude.ToString()
                    });
                }

                break;
            }
            else
            {
                Logger.LogInformation("Expected route {Expected} does not match.", prefRte);
            }
        }
        if (matchingRoute == null && prefRoutes.Any())
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                Type = ValidationMessage.ViolationType.NotRecommendedRoute,
                Param = string.Join(",", prefRoutes
                    .Where(r => !r.Remarks.Contains("AIP Route", StringComparison.InvariantCultureIgnoreCase))
                    .Select(r => r.RawRoute)),
            });
        }
        if (matchingRoute != null)
        {
            Messages.Add(new ValidationMessage
            {
                Field = ValidationMessage.FieldType.Route,
                Type = ValidationMessage.ViolationType.RouteMatchPreferred,
                Param = Flight.RawRoute ?? string.Empty,
            });
        }

        foreach (var (leg, index) in Legs.Select((l, i) => (l, i)))
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Validation cancelled.");
                return Messages;
            }
            if (leg.LegId == null && leg.LegIdentifier == "DCT"
                && leg.From.Type != FlightFix.FixType.Airport
                && leg.To.Type != FlightFix.FixType.Airport
                && matchingRoute == null)
            {
                var fromFix = await Navdata.GetFullQualifiedFixIdentifier(leg.From.Id, leg.From.Type switch
                {
                    FlightFix.FixType.Airport => INavdataProvider.FixType.Unknown,
                    FlightFix.FixType.Waypoint => INavdataProvider.FixType.Waypoint,
                    FlightFix.FixType.Vhf => INavdataProvider.FixType.Vhf,
                    FlightFix.FixType.Ndb => INavdataProvider.FixType.Ndb,
                    FlightFix.FixType.GeoCoord => INavdataProvider.FixType.Unknown,
                    FlightFix.FixType.Unknown => INavdataProvider.FixType.Unknown,
                    _ => throw new InvalidEnumArgumentException($"Unexpected fix type: {leg.From.Type}"),
                });
                var toFix = await Navdata.GetFullQualifiedFixIdentifier(leg.To.Id, leg.To.Type switch
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
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.Route,
                        FieldParam = index,
                        Type = ValidationMessage.ViolationType.Direct,
                    });
                }
            }

            if (leg.LegId != null)
            {
                var (fromLegId, toLegId) = leg.LegId.Value;
                var fromLeg = await Navdata.GetAirwayFix(fromLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {fromLegId}");
                var toLeg = await Navdata.GetAirwayFix(toLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {toLegId}");

                Logger.LogInformation("Validating airway leg {FromLeg} to {ToLeg} for flight {FlightId}",
                    fromLeg.FixIdentifier, toLeg.FixIdentifier, Flight.Id);
                Logger.LogInformation("Restrictions: From leg: {FromLeg}, To leg: {ToLeg}", fromLeg.DirectionalRestriction, toLeg.DirectionalRestriction);
                Logger.LogInformation("Sequence numbers: From leg: {FromSeq}, To leg: {ToSeq}",
                    fromLeg.SequenceNumber, toLeg.SequenceNumber);
                if (fromLeg.SequenceNumber <= toLeg.SequenceNumber && fromLeg.DirectionalRestriction == 'B' && matchingRoute == null)
                {
                    Logger.LogInformation("Violation found: From leg is backward.");
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.Route,
                        FieldParam = index,
                        Type = ValidationMessage.ViolationType.LegDirectionViolation,
                    });
                }
                // TODO: test for KARSI[5720] * - TR[5750] F is bidirectional
                if (toLeg.SequenceNumber <= fromLeg.SequenceNumber && toLeg.DirectionalRestriction == 'F' && matchingRoute == null)
                {
                    Logger.LogInformation("Violation found: To leg is forward.");
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.Route,
                        FieldParam = index,
                        Type = ValidationMessage.ViolationType.LegDirectionViolation,
                    });
                }

                if (fromLeg.FixIcaoCode.StartsWith("Z")
                    && toLeg.FixIcaoCode.StartsWith("Z")
                    && (fromLeg.AirwayIdentifier!.StartsWith("V") || fromLeg.AirwayIdentifier!.StartsWith("X"))
                    && matchingRoute == null)
                {
                    Messages.Add(new ValidationMessage
                    {
                        Field = ValidationMessage.FieldType.Route,
                        FieldParam = index,
                        Type = ValidationMessage.ViolationType.AirwayRequireApproval,
                    });
                }
            }
        }

        return Messages;
    }
}
