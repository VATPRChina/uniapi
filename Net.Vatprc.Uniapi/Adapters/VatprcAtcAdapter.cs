using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Adapters;

public class VatprcAtcApiAdapter(IOptions<VatprcAtcApiAdapter.Option> Options,
    IOptions<TokenService.Option> TokenOptions)
{
    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.ConfigureOptions<OptionConfigure>();
        builder.Services.AddSingleton<VatprcAtcApiAdapter>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Authentication:Internal:VatprcAtcService";

        public required string Endpoint { get; set; }

        public required string TokenAudience { get; set; }

        public required string PrivateKey { get; set; }

        public required string PublicKey { get; set; }

        public SecurityKey SecurityKey { get; set; } = null!;

        public SigningCredentials Credentials { get; set; } = null!;
    }

    public class OptionConfigure : IConfigureOptions<Option>
    {
        public void Configure(Option opts)
        {
            var key = ECDsa.Create();
            key.ImportFromPem(opts.PrivateKey);
            opts.SecurityKey = new ECDsaSecurityKey(key);
            opts.Credentials = new(opts.SecurityKey, opts.SecurityKey.KeySize switch
            {
                256 => SecurityAlgorithms.EcdsaSha256,
                384 => SecurityAlgorithms.EcdsaSha384,
                521 => SecurityAlgorithms.EcdsaSha512,
                _ => throw new NotSupportedException(),
            });
        }
    }

    public async Task<IEnumerable<Role>> GetUserRole(string cid)
    {
        var claims = new List<Claim>
        {
            new (JwtRegisteredClaimNames.Iat, DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString(), ClaimValueTypes.Integer64),
            new (JwtRegisteredClaimNames.Jti, Guid.NewGuid().ToString()),
        };
        var token = new JwtSecurityToken(
            issuer: TokenOptions.Value.Issuer,
            audience: Options.Value.TokenAudience,
            expires: DateTime.Now.Add(TimeSpan.FromMinutes(5)),
            notBefore: DateTime.Now,
            claims: claims,
            signingCredentials: Options.Value.Credentials);
        var tokenStr = new JwtSecurityTokenHandler().WriteToken(token);
        return await Options.Value.Endpoint
            .AppendPathSegments("v1/internal/users/", cid, "/roles")
            .WithHeader("User-Agent", UniapiUserAgent)
            .WithHeader("Token", tokenStr)
            .GetJsonAsync<IEnumerable<Role>>() ??
            throw new Exception("Unexpected null on fetch vatprc atcapi data");
    }

    public class Role
    {
        [JsonPropertyName("id")]
        public long Id { get; set; }

        [JsonPropertyName("name")]
        public string Name { get; set; } = string.Empty;

        /// <summary>
        /// Format: yyyy-MM-dd HH:mm:ss (UTC)
        /// </summary>
        [JsonPropertyName("expiration_time")]
        public string? ExpirationTime { get; set; }

        [JsonPropertyName("all_superroles")]
        public Role[] AllSuperroles { get; set; } = [];
    }
}
