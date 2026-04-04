using System.ComponentModel;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators;
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
        var planValidators = new IPlanValidator[]
        {
            new Validators.PlanValidators.Rnav1EquipmentValidator(),
            new Validators.PlanValidators.Rnav1PbnValidator(),
            new Validators.PlanValidators.RnpArValidator(),
            new Validators.PlanValidators.RvsmValidator(),
        };

        foreach (var pv in planValidators)
        {
            foreach (var m in pv.Validate(Flight))
            {
                Messages.Add(m);
            }
        }

        var prefRoutes = await Navdata.GetRecommendedRoutes(Flight.Departure, Flight.Arrival);
        Logger.LogInformation("Recommended routes for {Dep} to {Arr}: {Routes}",
            Flight.Departure, Flight.Arrival, prefRoutes.Select(r => r.RawRoute));

        PreferredRoute? matchingRoute = (await Task.WhenAll(prefRoutes.Select(async prefRte =>
        {
            var prefRteParsed = await RouteParserFactory.Create(prefRte.RawRoute, Navdata).Parse(ct);
            var matching = EnrouteRouteComparator.IsRouteMatchingExpected(Legs, prefRteParsed, loggerFactory.CreateLogger<EnrouteRouteComparator>(), ct);
            return new { PrefRoute = prefRte, IsMatching = matching };
        }))).Where(x => x.IsMatching).Select(x => x.PrefRoute).FirstOrDefault();
        Logger.LogInformation("Found matching route: {Route}", matchingRoute?.RawRoute);

        var preferredRouteValidators = new IPreferredRouteMatchValidator[]
        {
            new Validators.PreferredRouteValidators.HasMatchValidator(),
            new Validators.PreferredRouteValidators.CruisingLevelTypeValidator(),
            new Validators.PreferredRouteValidators.CruisingLevelAllowListValidator(),
        };
        foreach (var pv in preferredRouteValidators)
        {
            foreach (var m in pv.Validate(Flight, matchingRoute, prefRoutes))
            {
                Messages.Add(m);
            }
        }

        foreach (var (leg, index) in Legs.Select((l, i) => (l, i)))
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Validation cancelled.");
                return Messages;
            }

            AirwayFix? fromLeg = null;
            AirwayFix? toLeg = null;
            if (leg.LegId != null)
            {
                var (fromLegId, toLegId) = leg.LegId.Value;
                fromLeg = await Navdata.GetAirwayFix(fromLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {fromLegId}");
                toLeg = await Navdata.GetAirwayFix(toLegId)
                    ?? throw new InvalidOperationException($"Unexpected null airway leg: {toLegId}");
            }

            var legValidators = new ILegValidator[]
            {
                new Validators.LegValidators.DirectValidator(),
                new Validators.LegValidators.DirectionValidator(),
                new Validators.LegValidators.RestrictedAirwayValidator(),
            };

            foreach (var pv in legValidators)
            {
                await foreach (var m in pv.Validate(leg, index, Navdata, fromLeg, toLeg))
                {
                    Messages.Add(m);
                }
            }
        }

        return Messages;
    }
}
