using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;

namespace Net.Vatprc.Uniapi.Adapters;

public class VplaafAdapter
{
    protected const string ENDPOINT = "https://airspace.vplaaf.org/Areas.json";

    protected MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());

    public async Task<string> GetAreasAsync(CancellationToken ct = default)
    {
        return await Cache.GetOrCreateAsync("AREAS", entry =>
        {
            return ENDPOINT
                .WithHeader("User-Agent", UniapiUserAgent)
                .WithTimeout(15)
                .GetStringAsync(cancellationToken: ct);
        }) ?? throw new ApiError.InternalServerError(
            new InvalidOperationException("Unexpected null on fetching vPLAAF areas."));
    }
}
