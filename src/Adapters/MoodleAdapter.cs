using System.Diagnostics;
using Flurl;
using Flurl.Http;
using Flurl.Util;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Adapters.Moodle;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Adapters;

public interface IMoodleAdapter
{
    Task<IEnumerable<MoodleCreateUserResponseItem>> CreateUser(User user, CancellationToken ct = default);
    Task<MoodleUser?> GetUserByCid(string cid, CancellationToken ct = default);
}

public class MoodleAdapter(IOptions<MoodleAdapter.Option> options, ActivitySource activitySource) : IMoodleAdapter
{
    protected const string MOODLE_ENDPOINT = "https://moodle.vatprc.net/";

    public async Task<MoodleUser?> GetUserByCid(string cid, CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity($"{nameof(MoodleAdapter)}.{nameof(GetUserByCid)}", ActivityKind.Client);
        try
        {
            var result = await MOODLE_ENDPOINT
                .AppendPathSegment("webservice/rest/server.php")
                .PostUrlEncodedAsync(new Dictionary<string, string>()
                {
                    {"wstoken", options.Value.ApiKey},
                    {"wsfunction", "core_user_get_users_by_field"},
                    {"moodlewsrestformat", "json"},
                    {"field", "idnumber"},
                    {"values[0]", cid},
                }, cancellationToken: ct)
                .ReceiveJson<IEnumerable<MoodleUser>>();
            return result.SingleOrDefault();
        }
        catch (FlurlParsingException ex)
        {
            var error = await ex.GetResponseStringAsync();
            throw new Exception($"Moodle API error: {error}", ex);
        }
    }

    public async Task<IEnumerable<MoodleCreateUserResponseItem>> CreateUser(User user, CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity($"{nameof(MoodleAdapter)}.{nameof(CreateUser)}", ActivityKind.Client);
        try
        {
            var result = await MOODLE_ENDPOINT
            .AppendPathSegment("webservice/rest/server.php")
            .PostUrlEncodedAsync(new
            {
                wstoken = options.Value.ApiKey,
                wsfunction = "core_user_create_users",
                moodlewsrestformat = "json",
                users = new[]
                {
                    new
                    {
                        username = user.Cid,
                        idnumber = user.Cid,
                        createpassword = true,
                        firstname = user.FullName.SplitOnFirstOccurence(" ").FirstOrDefault() ?? user.Cid,
                        lastname = user.FullName.SplitOnFirstOccurence(" ").LastOrDefault() ?? user.Cid,
                        email = user.Email ?? $"{user.Cid}@noreply.users.vatprc.net",
                    },
                },
            }, cancellationToken: ct)
            .ReceiveJson<IEnumerable<MoodleCreateUserResponseItem>>();
            return result;
        }
        catch (FlurlParsingException ex)
        {
            var error = await ex.GetResponseStringAsync();
            throw new Exception($"Moodle API error: {error}", ex);
        }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<IMoodleAdapter, MoodleAdapter>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Moodle";

        public required string ApiKey { get; set; }
    }
}
