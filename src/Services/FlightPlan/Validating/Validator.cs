using System.ComponentModel;
using Net.Vatprc.Uniapi.Models.Acdm;
using Net.Vatprc.Uniapi.Models.Navdata;
using Net.Vatprc.Uniapi.Models.Navdata.Legs;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Services.FlightPlan.Validating.Validators;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Validating;

public class Validator(
    Flight flight,
    IList<Leg> legs,
    INavdataProvider navdata,
    RouteParserFactory routeParserFactory,
    ILogger<Validator> Logger,
    ILoggerFactory loggerFactory)
{
    protected readonly IList<ValidationMessage> messages = [];

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
            foreach (var m in pv.Validate(flight))
            {
                messages.Add(m);
            }
        }

        var prefRoutes = await navdata.GetRecommendedRoutes(flight.Departure, flight.Arrival);
        Logger.LogInformation("Recommended routes for {Dep} to {Arr}: {Routes}",
            flight.Departure, flight.Arrival, prefRoutes.Select(r => r.RawRoute));

        PreferredRoute? matchingRoute = null;
        foreach (var prefRte in prefRoutes)
        {
            var prefRteParsed = await routeParserFactory.Create(prefRte.RawRoute, navdata).Parse(ct);
            var matching = EnrouteRouteComparator.IsRouteMatchingExpected(legs, prefRteParsed, loggerFactory.CreateLogger<EnrouteRouteComparator>(), ct);
            if (matching)
            {
                matchingRoute = prefRte;
                Logger.LogInformation("Found matching route: {Route}", matchingRoute.RawRoute);
                break;
            }
        }
        Logger.LogInformation("Found matching route: {Route}", matchingRoute?.RawRoute);

        var preferredRouteValidators = new IPreferredRouteMatchValidator[]
        {
            new Validators.PreferredRouteValidators.HasMatchValidator(),
            new Validators.PreferredRouteValidators.CruisingLevelTypeValidator(),
            new Validators.PreferredRouteValidators.CruisingLevelAllowListValidator(),
        };
        foreach (var pv in preferredRouteValidators)
        {
            foreach (var m in pv.Validate(flight, matchingRoute, prefRoutes))
            {
                messages.Add(m);
            }
        }

        foreach (var (leg, index) in legs.Select((l, i) => (l, i)))
        {
            if (ct.IsCancellationRequested)
            {
                Logger.LogWarning("Validation cancelled.");
                return messages;
            }

            var legValidators = new ILegValidator[]
            {
                new Validators.LegValidators.DirectValidator(),
                new Validators.LegValidators.DirectionValidator(),
                new Validators.LegValidators.RestrictedAirwayValidator(),
            };

            foreach (var pv in legValidators)
            {
                await foreach (var m in pv.Validate(leg, index, navdata))
                {
                    messages.Add(m);
                }
            }
        }

        return messages;
    }
}
