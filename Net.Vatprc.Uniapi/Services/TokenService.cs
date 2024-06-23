using Microsoft.Extensions.Options;
using System.Security.Claims;
using System.IdentityModel.Tokens.Jwt;
using Microsoft.IdentityModel.Tokens;
using System.Security.Cryptography;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

public class TokenService(IOptionsMonitor<TokenService.Option> Options, IServiceScopeFactory Services)
{
    public TimeSpan FirstPartyExpires => Options.CurrentValue.FirstPartyExpires;

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.ConfigureOptions<OptionConfigure>();
        builder.Services.AddSingleton<TokenService>();
        return builder;
    }

    public string EncodeScopes(IEnumerable<string> scopes)
    {
        return string.Join(" ", scopes);
    }

    public IEnumerable<string> DecodeScopes(string scopes)
    {
        return scopes.Split(' ', StringSplitOptions.RemoveEmptyEntries);
    }

    public (string, JwtSecurityToken) IssueFirstParty(User user, Session refresh)
    {
        var claims = new List<Claim>
        {
            new (JwtRegisteredClaimNames.Sub, user.Id.ToString()),
            new (JwtRegisteredClaimNames.Iat, DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString(), ClaimValueTypes.Integer64),
            new (JwtRegisteredClaimNames.Jti, Guid.NewGuid().ToString()),
            new (JwtClaimNames.Scope, EncodeScopes(new[]{ JwtScopes.OAuth })),
            new (JwtClaimNames.UpdatedAt, user.UpdatedAt.ToUnixTimeSeconds().ToString(), ClaimValueTypes.Integer64),
            new (JwtRegisteredClaimNames.Sid, refresh.Token.ToString()),
            new (JwtClaimNames.ClientId, BuiltInClientId),
        };
        var token = new JwtSecurityToken(
            issuer: Options.CurrentValue.Issuer,
            audience: Options.CurrentValue.AudienceFirstParty,
            expires: DateTime.Now.Add(Options.CurrentValue.FirstPartyExpires),
            notBefore: DateTime.Now,
            claims: claims,
            signingCredentials: Options.CurrentValue.Credentials);
        return (new JwtSecurityTokenHandler().WriteToken(token), token);
    }

    public bool IsFirstParty(ClaimsPrincipal user)
    {
        return user.FindAll(JwtClaimNames.Scope)
                .Any(x => x.Value == JwtScopes.OAuth) &&
            user.FindAll(JwtRegisteredClaimNames.Aud)
                .Any(x => x.Value == Options.CurrentValue.AudienceFirstParty);
    }

    public async Task<Session> IssueFirstPartyRefreshToken(User user, Session? oldToken)
    {
        var now = DateTimeOffset.UtcNow;
        var expireTime = DateTimeOffset.UtcNow.Add(TimeSpan.FromDays(Options.CurrentValue.RefreshExpiresDays));
        var token = new Session
        {
            UserId = user.Id,
            UserUpdatedAt = user.UpdatedAt,
            Token = Ulid.NewUlid(),
            ExpiresIn = expireTime
        };
        using var services = Services.CreateScope();
        using var db = services.ServiceProvider.GetRequiredService<VATPRCContext>();
        using var transaction = db.Database.BeginTransaction();
        db.Session.Add(token);
        if (oldToken != null)
        {
            var oldTokenDb = await db.Session.FindAsync(oldToken.Token);
            db.Session.Remove(oldTokenDb!);
        }
        await db.SaveChangesAsync();
        await db.Session
            .Where(x => x.UserId == user.Id && x.ExpiresIn < now)
            .ExecuteDeleteAsync();
        await db.Session
            .Where(x => x.UserId == user.Id && x.UserUpdatedAt != user.UpdatedAt)
            .ExecuteDeleteAsync();
        await transaction.CommitAsync();
        return token;
    }

    public struct JwtClaimNames
    {
        /// <summary>
        /// https://datatracker.ietf.org/doc/html/rfc8693#name-scope-scopes-claim
        /// </summary>
        public const string Scope = "scope";
        /// <summary>
        /// https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims
        /// </summary>
        public const string UpdatedAt = "updated_at";
        /// <summary>
        /// https://www.rfc-editor.org/rfc/rfc8693.html#name-client_id-client-identifier
        /// </summary>
        public const string ClientId = "client_id";
    }

    public struct JwtScopes
    {
        public const string OAuth = "1p_oauth";
    }

    public const string BuiltInClientId = "vatprc";

    public class Option
    {
        public const string LOCATION = "Authentication:Jwt";

        public string PrivateKey { get; set; } = string.Empty;

        public string PublicKey { get; set; } = string.Empty;

        public string Issuer { get; set; } = string.Empty;

        public string AudienceFirstParty { get; set; } = string.Empty;

        public TimeSpan FirstPartyExpires { get; set; }

        public uint RefreshExpiresDays { get; set; }

        public SecurityKey SecurityKey { get; set; } = null!;

        public SigningCredentials Credentials { get; set; } = null!;

        public void BuildKeys()
        {

        }
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
}
