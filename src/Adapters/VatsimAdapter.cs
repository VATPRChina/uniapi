using System.Diagnostics;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;
using Net.Vatprc.Uniapi.Adapters.VatsimAdapterModels;

namespace Net.Vatprc.Uniapi.Adapters;

public class VatsimAdapter(IMemoryCache cache, ActivitySource activitySource)
{
    protected const string ONLINE_DATE_URL = "https://data.vatsim.net/v3/vatsim-data.json";
    public async Task<VatsimData> GetOnlineData(CancellationToken ct = default)
    {
        var result = await cache.GetOrCreateAsync(ONLINE_DATE_URL, async entry =>
        {
            using var activity = activitySource.StartActivity($"{nameof(VatsimAdapter)}.{nameof(GetOnlineData)}", ActivityKind.Client);
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await ONLINE_DATE_URL
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<VatsimData>(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatsim data");
        return result;
    }

    public async Task<UserInfo> GetUserInfo(string cid, CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity($"{nameof(VatsimAdapter)}.{nameof(GetUserInfo)}", ActivityKind.Client);
        return await "https://api.vatsim.net/v2/members/"
            .AppendPathSegment(cid)
            .WithHeader("User-Agent", UniapiUserAgent)
            .GetJsonAsync<UserInfo>(cancellationToken: ct)
            ?? throw new Exception("Unexpected null on fetch vatsim data");
    }

    protected const string DIVISION_EVENTS_URL = "https://my.vatsim.net/api/v2/events/view/division/PRC";
    public async Task<string> GetDivisionEventsAsString(CancellationToken ct = default)
    {
        var result = await cache.GetOrCreateAsync(DIVISION_EVENTS_URL, async entry =>
        {
            using var activity = activitySource.StartActivity($"{nameof(VatsimAdapter)}.{nameof(GetDivisionEventsAsString)}", ActivityKind.Client);
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await DIVISION_EVENTS_URL
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetStringAsync(cancellationToken: ct);
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }

    public async Task<string?> GetCidByDiscordUserId(string discordUserId, CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity($"{nameof(VatsimAdapter)}.{nameof(GetCidByDiscordUserId)}", ActivityKind.Client);
        try
        {
            var user = await "https://api.vatsim.net/v2/members/discord/"
                .AppendPathSegment(discordUserId)
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<DiscordUser>(cancellationToken: ct);
            return user.Cid;
        }
        catch (FlurlHttpException ex) when (ex.Call.Response?.StatusCode == 404)
        {
            return null;
        }
    }
}
