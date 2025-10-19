using Microsoft.Extensions.Caching.Memory;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class RouteParserFactory
{
    private readonly IMemoryCache _cache;

    public RouteParserFactory(IMemoryCache cache)
    {
        _cache = cache;
    }

    public RouteParser Create(string rawRoute, INavdataProvider navdata)
    {
        return new RouteParser(rawRoute, navdata, _cache);
    }
}
