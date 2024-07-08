using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class VatsimAuthService(IOptions<VatsimAuthService.Option> Options)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<VatsimAuthService>();
        return builder;
    }

    public class TokenResponse
    {
        [JsonPropertyName("token_type")]
        public required string TokenType { get; set; }

        [JsonPropertyName("expires_in")]
        public required int ExpiresIn { get; set; }

        [JsonPropertyName("access_token")]
        public required string AccessToken { get; set; }

        [JsonPropertyName("refresh_token")]
        public required string RefreshToken { get; set; }

        [JsonPropertyName("scopes")]
        public required IEnumerable<string> Scopes { get; set; }
    }

    public async Task<TokenResponse> GetTokenAsync(string code)
    {
        var response = await new Url(Options.Value.Endpoint)
            .AppendPathSegment("oauth/token")
            .PostUrlEncodedAsync(new
            {
                grant_type = "authorization_code",
                client_id = Options.Value.ClientId,
                client_secret = Options.Value.ClientSecret,
                redirect_uri = Options.Value.RedirectUri,
                code,
                code_verifier = "b727bef46e0b546964eaac18403d95e5bc284e9c1eab6ea48ae77a93",
            })
            .ReceiveJson<TokenResponse>();
        return response;
    }

    public class UserResponse
    {
        [JsonPropertyName("data")]
        public required DataObject Data { get; set; }

        public class DataObject
        {
            [JsonPropertyName("cid")]
            public required long Cid { get; set; }

            [JsonPropertyName("oauth")]
            public required OauthObject Oauth { get; set; }
        }

        public class OauthObject
        {
            [JsonPropertyName("token_valid")]
            public required bool TokenValid { get; set; }
        }
    }


    public async Task<UserResponse> GetUserAsync(string accessToken)
    {
        var response = await new Url(Options.Value.Endpoint)
            .AppendPathSegment("api/user")
            .WithOAuthBearerToken(accessToken)
            .GetJsonAsync<UserResponse>();
        return response;
    }

    public class Option
    {
        public const string LOCATION = "Authentication:Vatsim";

        public required string Endpoint { get; set; }

        public required string ClientId { get; set; }

        public required string ClientSecret { get; set; }

        public required string RedirectUri { get; set; }
    }
}
