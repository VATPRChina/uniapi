using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Net.Vatprc.Uniapi.Adapters.VatsimAdapterModels;

namespace Net.Vatprc.Uniapi.Adapters;

public class VatsimAdapter
{
    MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());

    public async Task<VatsimData> GetOnlineData(CancellationToken ct = default)
    {
        var result = await Cache.GetOrCreateAsync("vatsim-data", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://data.vatsim.net/v3/vatsim-data.json"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<VatsimData>(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatsim data");
        return result;
    }

    public async Task<UserInfo> GetUserInfo(string cid, CancellationToken ct = default)
    {
        return await "https://api.vatsim.net/v2/members/"
            .AppendPathSegment(cid)
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetJsonAsync<UserInfo>(cancellationToken: ct)
            ?? throw new Exception("Unexpected null on fetch vatsim data");
    }

    public async Task<IEnumerable<AtcSchedule>> GetAtcSchedule(CancellationToken ct = default)
    {
        var result = await Cache.GetOrCreateAsync("schedule", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://atcapi.vatprc.net/v1/public/schedules"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<IEnumerable<AtcSchedule>>(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }

    public async Task<IEnumerable<Controller>> GetAtcList(CancellationToken ct = default)
    {
        var result = await Cache.GetOrCreateAsync("controllers", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://atcapi.vatprc.net/v1/public/controllers"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<IEnumerable<Controller>>(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }

    public async Task<string> GetDivisionEventsAsString(CancellationToken ct = default)
    {
        var result = await Cache.GetOrCreateAsync("controllers", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://my.vatsim.net/api/v2/events/view/division/PRC"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetStringAsync(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }
}
