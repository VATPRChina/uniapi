using Flurl.Http;
using Microsoft.Extensions.Caching.Memory;

namespace Net.Vatprc.Uniapi.External;

public class TrackAudioService
{
    protected const string ENDPOINT = "https://raw.githubusercontent.com/pierr3/TrackAudio/main/MANDATORY_VERSION";

    protected MemoryCache Cache { get; init; } = new(new MemoryCacheOptions());

    public async Task<string> GetLastVersionAsync(CancellationToken ct = default)
    {
        return await Cache.GetOrCreateAsync("MANDATORY_VERSION", entry =>
        {
            return ENDPOINT
                .WithHeader("User-Agent", UniapiUserAgent)
                .WithTimeout(15)
                .GetStringAsync(cancellationToken: ct);
        }) ?? throw new ApiError.InternalServerError(
            new InvalidOperationException("Unexpected null on fetching TrackAudio version."));
    }
}
