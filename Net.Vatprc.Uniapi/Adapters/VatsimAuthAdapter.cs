using System.Security.Cryptography;
using System.Text;
using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Adapters;

public class VatsimAuthAdapter(IOptions<VatsimAuthAdapter.Option> Options)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<VatsimAuthAdapter>();
        return builder;
    }

    /// <summary>
    /// Generates a code_verifier and the corresponding code_challenge, as specified in the rfc-7636.
    /// </summary>
    /// <remarks>See https://datatracker.ietf.org/doc/html/rfc7636#section-4.1 and https://datatracker.ietf.org/doc/html/rfc7636#section-4.2</remarks>
    public static (string code_challenge, string verifier) GeneratePkce(int size = 32)
    {
        using var rng = RandomNumberGenerator.Create();
        var randomBytes = new byte[size];
        rng.GetBytes(randomBytes);
        var verifier = Base64UrlEncode(randomBytes);

        var buffer = Encoding.UTF8.GetBytes(verifier);
        var hash = SHA256.HashData(buffer);
        var challenge = Base64UrlEncode(hash);

        return (challenge, verifier);
    }

    private static string Base64UrlEncode(byte[] data) =>
        Convert.ToBase64String(data)
            .Replace("+", "-")
            .Replace("/", "_")
            .TrimEnd('=');

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

    public async Task<TokenResponse> GetTokenAsync(string code, string verifier)
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
                code_verifier = verifier,
                scope = "full_name email",
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
            public required string Cid { get; set; }

            [JsonPropertyName("oauth")]
            public required OauthObject Oauth { get; set; }

            [JsonPropertyName("personal")]
            public required PersonalObject Personal { get; set; }
        }

        public class OauthObject
        {
            [JsonPropertyName("token_valid")]
            public required string TokenValid { get; set; }
        }

        public class PersonalObject
        {
            [JsonPropertyName("name_first")]
            public required string FirstName { get; set; }

            [JsonPropertyName("name_last")]
            public required string LastName { get; set; }

            [JsonPropertyName("name_full")]
            public required string FullName { get; set; }

            [JsonPropertyName("email")]
            public required string Email { get; set; }
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
