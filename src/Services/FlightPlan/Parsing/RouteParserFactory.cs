using Microsoft.Extensions.Caching.Memory;

namespace Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;

public class RouteParserFactory(IMemoryCache cache, ILoggerFactory loggerFactory)
{
    public RouteParser Create(string rawRoute, INavdataProvider navdata)
    {
        return new RouteParser(rawRoute, navdata, cache, loggerFactory);
    }
}
