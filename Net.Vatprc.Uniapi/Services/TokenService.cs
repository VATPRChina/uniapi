using Microsoft.Extensions.Options;
using System.Security.Claims;
using System.IdentityModel.Tokens.Jwt;
using Microsoft.IdentityModel.Tokens;
using System.Security.Cryptography;
using Net.Vatprc.Uniapi.Models;

namespace Net.Vatprc.Uniapi.Services;

public class TokenService(IOptionsMonitor<TokenService.Option> Options, IServiceScopeFactory Services)
{
    public TimeSpan AccessTokenExpires => Options.CurrentValue.FirstPartyExpires;
    public TimeSpan DeviceAuthzExpires => TimeSpan.FromSeconds(Options.CurrentValue.DeviceAuthzExpires);

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

    public (string, JwtSecurityToken) IssueAccessToken(User user, RefreshToken refresh)
    {
        var claims = new List<Claim>
        {
            new (JwtRegisteredClaimNames.Sub, user.Id.ToString()),
            new (JwtRegisteredClaimNames.Iat, DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString(), ClaimValueTypes.Integer64),
            new (JwtRegisteredClaimNames.Jti, Guid.NewGuid().ToString()),
            new (JwtClaimNames.Scope, EncodeScopes([])),
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

    public async Task<RefreshToken> IssueRefreshToken(User user, RefreshToken? oldToken = null, bool createCode = false)
    {
        var now = DateTimeOffset.UtcNow;
        var expireTime = DateTimeOffset.UtcNow.Add(TimeSpan.FromDays(Options.CurrentValue.RefreshExpiresDays));
        var token = new RefreshToken
        {
            UserId = user.Id,
            UserUpdatedAt = user.UpdatedAt,
            Token = Ulid.NewUlid(),
            ExpiresIn = expireTime,
            AuthzCode = createCode ? Ulid.NewUlid() : null,
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

    public string GenerateAuthCode(RefreshToken session, string clientId, string redirectUri)
    {
        var code = session.AuthzCode.ToString() ?? throw new ArgumentException("Session code is empty");
        var claims = new List<Claim>
        {
            new (JwtRegisteredClaimNames.Iat, DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString(), ClaimValueTypes.Integer64),
            new (JwtClaimNames.ClientId, clientId),
            new (JwtClaimNames.RedirectUri, redirectUri),
            new (JwtClaimNames.AuthCode, code),
            new (JwtRegisteredClaimNames.Jti, code),
        };
        var token = new JwtSecurityToken(
            issuer: Options.CurrentValue.Issuer,
            audience: Options.CurrentValue.AudienceFirstParty,
            expires: DateTime.Now.Add(Options.CurrentValue.FirstPartyExpires),
            notBefore: DateTime.Now,
            claims: claims,
            signingCredentials: Options.CurrentValue.Credentials);
        return new JwtSecurityTokenHandler().WriteToken(token);
    }

    public class InvalidClientIdOrRedirectUriException : Exception
    {
        public InvalidClientIdOrRedirectUriException() : base("invalid client_id or redirect_uri") { }
    }

    public async Task<RefreshToken?> GetRefreshTokenByCode(string code, string clientId, string? redirectUri = null)
    {
        var claims = new JwtSecurityTokenHandler().ValidateToken(code, new TokenValidationParameters
        {
            IssuerSigningKey = Options.CurrentValue.SecurityKey,
            ValidIssuer = Options.CurrentValue.Issuer,
            ValidAudience = Options.CurrentValue.AudienceFirstParty,
            ValidateIssuer = true,
            ValidateAudience = true,
            ValidateLifetime = true,
            ValidateIssuerSigningKey = true,
        }, out var token);
        if (claims.FindFirstValue(JwtClaimNames.ClientId) != clientId ||
            (!string.IsNullOrEmpty(redirectUri) && claims.FindFirstValue(JwtClaimNames.RedirectUri) != redirectUri))
        {
            throw new InvalidClientIdOrRedirectUriException();
        }
        using var services = Services.CreateScope();
        using var db = services.ServiceProvider.GetRequiredService<VATPRCContext>();
        var session = await db.Session
            .Include(x => x.User)
            .FirstOrDefaultAsync(x => x.AuthzCode == Ulid.Parse(token.Id));
        if (session != null)
        {
            session.AuthzCode = null;
            await db.SaveChangesAsync();
        }
        return session;
    }

    public bool CheckClientExists(string clientId)
    {
        return Options.CurrentValue.Clients
            .Any(x => x.ClientId == clientId);
    }

    public bool CheckClientExists(string clientId, string redirectUri)
    {
        return Options.CurrentValue.Clients
            .Any(x => x.ClientId == clientId && x.RedirectUri.Contains(redirectUri));
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
        /// <summary>
        /// redirect uri, only used in auth code
        /// </summary>
        public const string RedirectUri = "redirect_uri";
        /// <summary>
        /// redirect uri, only used in auth code
        /// </summary>
        public const string AuthCode = "authorization_code";
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

        /// <summary>
        /// Expiration of device authorization in seconds.
        /// </summary>
        public uint DeviceAuthzExpires { get; set; }

        public SecurityKey SecurityKey { get; set; } = null!;

        public SigningCredentials Credentials { get; set; } = null!;

        public IEnumerable<Client> Clients { get; set; } = [];

        public class Client
        {
            public string ClientId { get; set; } = string.Empty;

            public IEnumerable<string> RedirectUri { get; set; } = [];
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
