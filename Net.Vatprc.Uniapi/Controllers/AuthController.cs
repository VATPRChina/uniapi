using System.Text.Json.Serialization;
using Flurl;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Http.Extensions;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;
using Swashbuckle.AspNetCore.Annotations;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("auth")]
[AllowAnonymous]
[ResponseCache(NoStore = true)]
public class AuthController(
    IOptions<VatsimAuthService.Option> Options,
    TokenService TokenService,
    VatsimAuthService AuthService,
    VATPRCContext DbContext,
    ILogger<AuthController> Logger) : ControllerBase
{
    protected void ClearCookies()
    {
        Logger.LogInformation("Clear cookies: client_id, redirect, user_code, code_verifier");
        Response.Cookies.Delete("client_id");
        Response.Cookies.Delete("redirect");
        Response.Cookies.Delete("user_code");
        Response.Cookies.Delete("code_verifier");
    }

    /// <summary>
    /// Authorization request
    /// </summary>
    /// <param name="response_type">REQUIRED.  Value MUST be set to "code".</param>
    /// <param name="client_id">REQUIRED.  The client identifier as described in Section 2.2.</param>
    /// <param name="redirect_uri">OPTIONAL.  As described in Section 3.1.2.</param>
    /// <returns></returns>
    /// <see href="https://datatracker.ietf.org/doc/html/rfc6749#autoid-36" />
    [HttpGet("authorize")]
    [ProducesResponseType(StatusCodes.Status307TemporaryRedirect)]
    public IActionResult Authorize(
        [FromQuery] string response_type,
        [FromQuery] string client_id,
        [FromQuery] string redirect_uri)
    {
        ClearCookies();
        Logger.LogInformation("Authorize code flow: {response_type}, {client_id}, {redirect_uri}",
            response_type, client_id, redirect_uri);
        if (response_type != "code")
        {
            return BadRequest("invalid response_type");
        }
        if (!TokenService.CheckClientExists(client_id, redirect_uri))
        {
            return Unauthorized("client is invalid");
        }
        Logger.LogInformation("Add cookie client_id: {client_id}, redirect: {redirect_uri}",
            client_id, redirect_uri);
        Response.Cookies.Append("client_id", client_id, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
            Expires = DateTimeOffset.UtcNow + TimeSpan.FromHours(1),
        });
        Response.Cookies.Append("redirect", redirect_uri, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
            Expires = DateTimeOffset.UtcNow + TimeSpan.FromHours(1),
        });
        Logger.LogInformation("Redirect to login");
        return RedirectToActionPreserveMethod(nameof(Login));
    }

    /// <summary>
    /// Device Authorization Response
    /// </summary>
    /// <see href="https://datatracker.ietf.org/doc/html/rfc8628#section-3.2"/>
    public record DeviceAuthorizationResponse
    {
        /// <summary>
        /// REQUIRED.  The device verification code.
        /// </summary>
        public required string DeviceCode { get; set; }
        /// <summary>
        /// REQUIRED.  The end-user verification code.
        /// </summary>
        public required string UserCode { get; set; }
        /// <summary>
        /// REQUIRED.  The end-user verification URI on the authorization
        /// server.  The URI should be short and easy to remember as end users
        /// will be asked to manually type it into their user agent.
        /// </summary>
        public required string VerificationUri { get; set; }
        /// <summary>
        /// OPTIONAL.  A verification URI that includes the "user_code" (or
        /// other information with the same function as the "user_code"),
        /// which is designed for non-textual transmission.
        /// </summary>
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public string? VerificationUriComplete { get; set; }
        /// <summary>
        /// REQUIRED.  The lifetime in seconds of the "device_code" and
        /// "user_code".
        /// </summary>
        public required uint ExpiresIn { get; set; }
        /// <summary>
        /// OPTIONAL.  The minimum amount of time in seconds that the client
        /// SHOULD wait between polling requests to the token endpoint.  If no
        /// value is provided, clients MUST use 5 as the default.
        /// </summary>
        [JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
        public uint? Interval { get; set; }
    };

    protected const string USER_CODE_ALPHBET = "BCDFGHJKLMNPQRSTVWXZ";

    /// <summary>
    /// Device Authorization Request
    /// </summary>
    /// <see href="https://datatracker.ietf.org/doc/html/rfc8628#section-3.2"/>
    public record DeviceAuthzRequest
    {
        /// <summary>
        /// REQUIRED if the client is not authenticating with the
        /// authorization server as described in Section 3.2.1. of [RFC6749].
        /// The client identifier as described in Section 2.2 of [RFC6749].
        /// </summary>
        public required string client_id { get; set; }
        /// <summary>
        /// OPTIONAL.  The scope of the access request as defined by
        /// Section 3.3 of [RFC6749].
        /// </summary>
        public string? scope { get; set; }
    }

    /// <summary>
    /// Device authorization
    /// </summary>
    /// <param name="req"></param>
    /// <returns></returns>
    /// <see href="https://datatracker.ietf.org/doc/html/rfc8628" />
    [HttpPost("device_authorization")]
    [Consumes("application/x-www-form-urlencoded")]
    [ProducesResponseType(typeof(TokenResponse), StatusCodes.Status200OK)]
    [ProducesResponseType(typeof(TokenErrorDto), StatusCodes.Status400BadRequest)]
    public async Task<IActionResult> DeviceAuthorization([FromForm] DeviceAuthzRequest req)
    {
        Logger.LogInformation("Device code flow");

        if (!TokenService.CheckClientExists(req.client_id))
        {
            return Unauthorized(new TokenErrorDto
            {
                Error = "invalid_client",
                ErrorDescription = "client_id not found",
            });
        }

        var random = new Random();
        var deviceAuthz = new DeviceAuthorization
        {
            DeviceCode = Ulid.NewUlid(),
            UserCode = new string(Enumerable.Range(1, 8)
                .Select(_ => USER_CODE_ALPHBET[random.Next(USER_CODE_ALPHBET.Length)])
                .ToArray()),
            ExpiresAt = DateTimeOffset.UtcNow + TokenService.DeviceAuthzExpires,
            ClientId = req.client_id,
        };
        DbContext.DeviceAuthorization.Add(deviceAuthz);
        await DbContext.SaveChangesAsync();
        Logger.LogInformation("Init device code: {device_code}, user_code: {user_code}",
            deviceAuthz.DeviceCode, deviceAuthz.UserCode);

        var userCode = deviceAuthz.UserCode[..4] + "-" + deviceAuthz.UserCode[4..];
        var url = new Uri(Request.GetEncodedUrl())
            .GetLeftPart(UriPartial.Authority)
            .AppendPathSegment(Url.Action(nameof(Device)));
        return Ok(new DeviceAuthorizationResponse
        {
            DeviceCode = deviceAuthz.DeviceCode.ToString(),
            UserCode = userCode,
            VerificationUri = url,
            VerificationUriComplete = url
                .SetQueryParam("user_code", deviceAuthz.UserCode),
            ExpiresIn = Convert.ToUInt32((deviceAuthz.ExpiresAt - DateTimeOffset.UtcNow).TotalSeconds),
        });
    }

    [HttpGet("device")]
    [ApiExplorerSettings(IgnoreApi = true)]
    public async Task<IActionResult> Device([FromQuery] string user_code)
    {
        ClearCookies();
        var deviceAuthz = await DbContext.DeviceAuthorization
            .FirstOrDefaultAsync(x => x.UserCode == user_code);
        if (deviceAuthz == null)
        {
            return NotFound();
        }
        if (deviceAuthz.UserId != null)
        {
            DbContext.Remove(deviceAuthz);
            await DbContext.SaveChangesAsync();
            return BadRequest("Already used.");
        }
        if (deviceAuthz.IsExpired)
        {
            DbContext.Remove(deviceAuthz);
            await DbContext.SaveChangesAsync();
            return BadRequest("Device code expired");
        }
        Response.Cookies.Append("user_code", user_code, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
            Expires = deviceAuthz.ExpiresAt,
        });
        return RedirectToActionPreserveMethod(nameof(Login));
    }

    [HttpGet("login")]
    [ApiExplorerSettings(IgnoreApi = true)]
    public IActionResult Login()
    {
        var (challenge, verifier) = VatsimAuthService.GeneratePkce();
        var url = new Url(Options.Value.Endpoint)
            .AppendPathSegment("oauth/authorize")
            .SetQueryParam("response_type", "code")
            .SetQueryParam("client_id", Options.Value.ClientId)
            .SetQueryParam("redirect_uri", Options.Value.RedirectUri)
            .SetQueryParam("code_challenge", challenge)
            .SetQueryParam("code_challenge_method", "S256")
            .SetQueryParam("scope", "full_name email");
        Response.Cookies.Append("code_verifier", verifier, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
            Expires = DateTimeOffset.UtcNow + TimeSpan.FromHours(1),
        });
        return RedirectPreserveMethod(url.ToString());
    }

    [HttpGet("callback/vatsim")]
    [ApiExplorerSettings(IgnoreApi = true)]
    public async Task<IActionResult> VatsimCallback(string? code, string? state, string? error)
    {
        if (string.IsNullOrEmpty(code))
        {
            return BadRequest("Missing code");
        }

        Logger.LogInformation("Recevied VATSIM authn callback");

        Request.Cookies.TryGetValue("code_verifier", out var verifier);
        Response.Cookies.Delete("code_verifier");
        Logger.LogInformation("Delete code_verifier cookie");

        var token = await AuthService.GetTokenAsync(code, verifier ?? string.Empty);
        var vatsimUser = await AuthService.GetUserAsync(token.AccessToken);

        var user = await DbContext.User.FirstOrDefaultAsync(x => x.Cid == vatsimUser.Data.Cid);
        if (user == null)
        {
            user = new()
            {
                Cid = vatsimUser.Data.Cid,
            };
            DbContext.User.Add(user);
            Logger.LogInformation("Create new user: cid={cid}", user.Cid);
        }
        user.FullName = vatsimUser.Data.Personal.FullName;
        user.Email = vatsimUser.Data.Personal.Email;
        await DbContext.SaveChangesAsync();

        if (Request.Cookies.TryGetValue("redirect", out var redirect)
            && !string.IsNullOrEmpty(redirect)
            && Request.Cookies.TryGetValue("client_id", out var clientId)
            && !string.IsNullOrEmpty(clientId)
            && Uri.TryCreate(redirect, UriKind.Absolute, out var redirectUri))
        {
            Logger.LogInformation("Found cookie, redirect to client: {client_id}, {redirect_uri}",
                clientId, redirect);
            Response.Cookies.Delete("client_id");
            Response.Cookies.Delete("redirect");
            var refresh = await TokenService.IssueFirstPartyRefreshToken(user, createCode: true);
            var authCode = TokenService.GenerateAuthCode(refresh, clientId, redirect.ToString());
            var redirectTarget = redirectUri
                .SetQueryParam("code", authCode)
                .ToString();
            return RedirectPreserveMethod(redirectTarget);
        }
        else if (Request.Cookies.TryGetValue("user_code", out var user_code)
            && !string.IsNullOrEmpty(user_code))
        {
            Logger.LogInformation("Found cookie, user_code={user_code}", user_code);
            var deviceAuthz = await DbContext.DeviceAuthorization
                .FirstOrDefaultAsync(x => x.UserCode == user_code);
            if (deviceAuthz == null)
            {
                return Unauthorized("Device authorization not found");
            }
            deviceAuthz.UserId = user.Id;
            Logger.LogInformation("Associated user: {user_id} to device: {device_code}",
                user.Id, deviceAuthz.DeviceCode);
            await DbContext.SaveChangesAsync();

            Response.Cookies.Delete("user_code");
            Logger.LogInformation("Delete user_code cookie");

            return Ok("Login successful, please return to your device.");
        }

        return Ok(new
        {
            VatsimUser = vatsimUser,
            User = user,
            Code = code,
            Token = token,
            State = state,
        });
    }

    /// <summary>
    /// Access Token Request
    /// </summary>
    [SwaggerDiscriminator("GrantType")]
    [SwaggerSubType(typeof(DeviceAccessTokenRequest), DiscriminatorValue = "urn:ietf:params:oauth:grant-type:device_code")]
    [SwaggerSubType(typeof(RefreshAccessTokenRequest), DiscriminatorValue = "refresh_token")]
    public record AccessTokenRequest
    {
        /// <summary>
        /// REQUIRED. Identifier of the grant type the client uses
        /// with the particular token request. This specification defines the
        /// values authorization_code, refresh_token, and client_credentials.
        /// The grant type determines the further parameters required or
        /// supported by the token request.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-3.2.2" />
        [ModelBinder(Name = "grant_type")]
        public string GrantType { get; set; } = string.Empty;

        /// <summary>
        /// REQUIRED, if the client is not authenticating with the
        /// authorization server as described in Section 3.2.1.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-3.2.2" />
        // FIXME: This is not working
        [ModelBinder(Name = "client_id")]
        public string ClientId { get; set; } = string.Empty;
    }

    public record DeviceAccessTokenRequest
    {
        /// <summary>
        /// REQUIRED.  The device verification code, "device_code" from the
        /// device authorization response, defined in Section 3.2.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/rfc8628#section-3.5"/>
        [ModelBinder(Name = "device_code")]
        public string DeviceCode { get; set; } = string.Empty;
    }

    public record RefreshAccessTokenRequest
    {
        /// <summary>
        /// REQUIRED. The refresh token issued to the client.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-4.3.1"/>
        [ModelBinder(Name = "refresh_token")]
        public string RefreshToken { get; set; } = string.Empty;
    }

    /// <summary>
    /// Token response, compliant with RFC 8693
    /// </summary>
    /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-3.2.3"/>
    /// <example>
    /// {
    ///     "access_token": "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIwMUoyOEJWWjlaVkpWQjBWUTEwMTQwSEU1OSIsImlhdCI6MTcyNzE1OTc5MywianRpIjoiODkxODEyMjYtN2M5ZS00NmMzLWJmNTMtZWY3MTMzNTkyMTRlIiwic2NvcGUiOiIiLCJ1cGRhdGVkX2F0IjoxNzIwNDE1Mjg4LCJzaWQiOiIwMUo4SEJYWEVYUUU3UzUyMEJURUdDR1FCTSIsImNsaWVudF9pZCI6InZhdHByYyIsIm5iZiI6MTcyNzE1OTc5MywiZXhwIjoxNzI3MTYzMzkzLCJpc3MiOiJodHRwczovL3VuaWFwaS52YXRwcmMubmV0IiwiYXVkIjoiaHR0cHM6Ly91bmlhcGkudmF0cHJjLm5ldCJ9.mtYoT9kbOyTF-Xk-h2mNBWYPyiOFwmH_v4U1qz16tt-beJ0RxTGFa4x6yr117r4w0Wy0AQLHWo1_5icG3IgigQ",
    ///     "token_type": "Bearer",
    ///     "expires_in": 3600,
    ///     "refresh_token": "01J8HBXXEXQE7S520BTEGCGQBM",
    ///     "scope": ""
    /// }
    /// </example>
    public record TokenResponse
    {
        /// <summary>
        /// REQUIRED.  The access token issued by the authorization server.
        /// </summary>
        public required string AccessToken { get; set; }
        /// <summary>
        /// REQUIRED.  The type of the token issued as described in
        /// Section 7.1.  Value is case insensitive.
        /// </summary>
        public string TokenType { get; set; } = "Bearer";
        /// <summary>
        /// RECOMMENDED.  The lifetime in seconds of the access token.  For
        /// example, the value "3600" denotes that the access token will
        /// expire in one hour from the time the response was generated.
        /// If omitted, the authorization server SHOULD provide the
        /// expiration time via other means or document the default value.
        /// </summary>
        public required uint ExpiresIn { get; set; }
        /// <summary>
        /// OPTIONAL.  The refresh token, which can be used to obtain new
        /// access tokens using the same authorization grant as described
        /// in Section 6.
        /// </summary>
        public required string RefreshToken { get; set; }
        /// <summary>
        /// OPTIONAL, if identical to the scope requested by the client;
        /// otherwise, REQUIRED.  The scope of the access token as
        /// described by Section 3.3.
        /// </summary>
        public required string Scope { get; set; }
    };

    public record TokenErrorDto
    {
        /// <summary>
        /// REQUIRED.  A single ASCII [USASCII] error code.
        /// </summary>
        public string Error { get; set; } = string.Empty;
        /// <summary>
        /// OPTIONAL.  Human-readable ASCII [USASCII] text providing
        /// additional information, used to assist the client developer in
        /// understanding the error that occurred.
        /// Values for the "error_description" parameter MUST NOT include
        /// characters outside the set %x20-21 / %x23-5B / %x5D-7E.
        /// </summary>
        public string? ErrorDescription { get; set; }
        /// <summary>
        /// OPTIONAL.  A URI identifying a human-readable web page with
        /// information about the error, used to provide the client
        /// developer with additional information about the error.
        /// Values for the "error_uri" parameter MUST conform to the
        /// URI-reference syntax and thus MUST NOT include characters
        /// outside the set %x21 / %x23-5B / %x5D-7E.
        /// </summary>
        public string? ErrorUri { get; set; }
        /// <summary>
        /// REQUIRED if a "state" parameter was present in the client
        /// authorization request.  The exact value received from the
        /// client.
        /// </summary>
        public string? State { get; set; }
    }

    /// <summary>
    /// Get token
    /// </summary>
    /// <param name="req"></param>
    /// <param name="deviceReq"></param>
    /// <param name="refreshReq"></param>
    /// <returns></returns>
    [HttpPost("token")]
    [Consumes("application/x-www-form-urlencoded")]
    [ProducesResponseType(typeof(TokenResponse), StatusCodes.Status200OK)]
    [ProducesResponseType(typeof(TokenErrorDto), StatusCodes.Status400BadRequest)]
    public async Task<IActionResult> Token(
        [FromForm] AccessTokenRequest req,
        [FromForm] DeviceAccessTokenRequest deviceReq,
        [FromForm] RefreshAccessTokenRequest refreshReq)
    {
        if (req.GrantType == "urn:ietf:params:oauth:grant-type:device_code")
        {
            return await DeviceCodeGrant(req, deviceReq);
        }
        if (req.GrantType == "refresh_token")
        {
            return await RefreshTokenGrant(refreshReq);
        }
        else
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "unsupported_grant_type",
                ErrorDescription = "The authorization grant type is not supported by the authorization server.",
            });
        }
    }

    protected async Task<IActionResult> DeviceCodeGrant(AccessTokenRequest tokenReq, DeviceAccessTokenRequest req)
    {
        if (string.IsNullOrWhiteSpace(req.DeviceCode))
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_request",
                ErrorDescription = "Missing device code",
            });
        }

        if (!Ulid.TryParse(req.DeviceCode, out var deviceCode))
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Device code not found",
            });
        }
        var deviceAuthz = await DbContext.DeviceAuthorization
            .Include(x => x.User)
            .FirstOrDefaultAsync(x => x.DeviceCode == deviceCode);
        if (deviceAuthz == null)
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Device code not found",
            });
        }
        if (deviceAuthz.IsExpired)
        {
            DbContext.DeviceAuthorization.Remove(deviceAuthz);
            await DbContext.SaveChangesAsync();
            return BadRequest(new TokenErrorDto
            {
                Error = "expired_token",
                ErrorDescription = "Device code expired",
            });
        }
        if (deviceAuthz.User == null)
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "authorization_pending",
                ErrorDescription = "User has not yet authorized this device",
            });
        }
        if (deviceAuthz.ClientId != tokenReq.ClientId)
        {
            Logger.LogInformation("Client ID mismatch: req {client_id} != db {device_client_id}",
                tokenReq.ClientId, deviceAuthz.ClientId);
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_client",
                ErrorDescription = "Client ID mismatch",
            });
        }

        var refresh = await TokenService.IssueFirstPartyRefreshToken(deviceAuthz.User, null);
        var (token, jwt) = TokenService.IssueFirstParty(deviceAuthz.User, refresh);
        var expires = jwt.Payload.Expiration ?? 0;
        var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        var scopes = jwt.Payload.Claims.FirstOrDefault(x => x.Type == TokenService.JwtClaimNames.Scope)?.Value ?? "";
        Logger.LogInformation("Issued token for user: {user_id}, expires: {expires}, scopes: {scopes}",
            deviceAuthz.User.Id, expires, scopes);

        Logger.LogInformation("Remove device authorization {deviceCode}", deviceCode);
        DbContext.DeviceAuthorization.Remove(deviceAuthz);
        await DbContext.SaveChangesAsync();

        return Ok(new TokenResponse
        {
            AccessToken = token,
            ExpiresIn = (uint)(expires - now),
            RefreshToken = refresh.Token.ToString(),
            Scope = scopes
        });
    }

    protected async Task<IActionResult> RefreshTokenGrant(RefreshAccessTokenRequest req)
    {
        if (string.IsNullOrWhiteSpace(req.RefreshToken))
        {
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_request",
                ErrorDescription = "Missing refresh token",
            });
        }

        if (!Ulid.TryParse(req.RefreshToken, out var tokenId))
        {
            Logger.LogInformation("Refresh token is not ULID");
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Refresh token not found",
            });
        }

        var refresh = await DbContext.Session
            .Include(x => x.User)
            .FirstOrDefaultAsync(x => x.Token == tokenId);
        if (refresh == null)
        {
            Logger.LogInformation("Refresh token not found");
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Refresh token not found",
            });
        }

        if (refresh.UserUpdatedAt != refresh.User.UpdatedAt)
        {
            Logger.LogInformation("User updated, revoke refresh token");
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Refresh token has been revoked",
            });
        }
        if (refresh.ExpiresIn < DateTimeOffset.Now)
        {
            Logger.LogInformation("Refresh token expired");
            return BadRequest(new TokenErrorDto
            {
                Error = "invalid_grant",
                ErrorDescription = "Refresh token expired",
            });
        }
        var newRefresh = await TokenService.IssueFirstPartyRefreshToken(refresh.User, refresh);
        var (token, jwt) = TokenService.IssueFirstParty(refresh.User, newRefresh);
        var expires = jwt.Payload.Expiration ?? 0;
        var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
        var scopes = jwt.Payload.Claims.FirstOrDefault(x => x.Type == TokenService.JwtClaimNames.Scope)?.Value ?? "";

        return Ok(new TokenResponse
        {
            AccessToken = token,
            ExpiresIn = (uint)(expires - now),
            RefreshToken = newRefresh.Token.ToString(),
            Scope = scopes
        });
    }
}
