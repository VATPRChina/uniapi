using System.Diagnostics;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;

namespace Net.Vatprc.Uniapi.Services;

public class VatsimService
{
    MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.AddSingleton<VatsimService>();
        return builder;
    }

    public async Task<VatsimData.VatsimData> GetOnlineData()
    {
        var result = await Cache.GetOrCreateAsync("vatsim-data", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://data.vatsim.net/v3/vatsim-data.json"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<VatsimData.VatsimData>();
        }) ?? throw new Exception("Unexpected null on fetch vatsim data");
        return result;
    }

    public async Task<IEnumerable<VatsimData.AtcSchedule>> GetAtcSchedule()
    {
        var result = await Cache.GetOrCreateAsync("schedule", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://atcapi.vatprc.net/v1/public/schedules"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<IEnumerable<VatsimData.AtcSchedule>>();
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }

    public async Task<IEnumerable<VatsimData.Controller>> GetAtcList()
    {
        var result = await Cache.GetOrCreateAsync("controllers", async entry =>
        {
            entry.AbsoluteExpirationRelativeToNow = TimeSpan.FromMinutes(1);
            return await "https://atcapi.vatprc.net/v1/public/controllers"
                .WithHeader("User-Agent", UniapiUserAgent)
                .GetJsonAsync<IEnumerable<VatsimData.Controller>>();
        }) ?? throw new Exception("Unexpected null on fetch vatprc data");
        return result;
    }
}
