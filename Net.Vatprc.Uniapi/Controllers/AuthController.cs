using System.Text.Json.Serialization;
using Flurl;
using Flurl.Http;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Http.Extensions;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("auth")]
[AllowAnonymous]
[ResponseCache(NoStore = true)]
public class AuthController(
    IOptions<VatsimAuthService.Option> Options,
    TokenService TokenService,
    VatsimAuthService AuthService,
    VATPRCContext DbContext,
    ILogger<AuthController> Logger) : Controller
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
    [ProducesResponseType(typeof(DeviceAuthorizationResponse), StatusCodes.Status200OK)]
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
            .AppendPathSegment(Url.Action(nameof(DeviceConfirm)));
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

    protected string NormalizeUserCode(string? user_code)
    {
        return string.Concat((user_code ?? "").ToUpper().Where(USER_CODE_ALPHBET.Contains));
    }

    protected IActionResult RenderDeviceCodeUI(string? user_code)
    {
        var code = NormalizeUserCode(user_code);
        if (code.Length != 8)
        {
            return Content($"""
                <!doctype html>
                <html>
                <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <link href="/style.css" rel="stylesheet">
                </head>
                <body class="grid h-screen place-items-center bg-slate-100">
                <form class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 flex flex-col gap-y-2">
                    <h1 class="text-4xl font-bold">Device Code Login</h1>
                    {(user_code != null ? $"""<h2 class="text-xl text-red-700">The provided code <span class="font-mono">{user_code.ToUpper()}</span> is invalid.</h2>""" : "")}
                    <h2 class="text-2xl">Please type your code as on your device.</h2>
                    <input class="my-4 border-2 rounded-md text-3xl font-bold text-center uppercase" type="text" name="user_code" required placeholder="BCDF-GHJK" >
                    <button type="submit" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Proceed</button>
                </form>
                </body>
                </html>
                """, "text/html");
        }
        return Content($"""
            <!doctype html>
            <html>
            <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link href="/style.css" rel="stylesheet">
            </head>
            <body class="grid h-screen place-items-center bg-slate-100">
            <form class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 flex flex-col gap-y-2">
                <h1 class="text-4xl font-bold">Device Code Login</h1>
                <h2 class="text-2xl">Please check if the following code matches your device.</h2>
                <div><div class="text-3xl font-bold w-fit mx-auto my-4">{code[..4]}-{code[4..]}</div></div>
                <input type="hidden" name="user_code" value="{user_code}">
                <input type="hidden" name="confirm" value="true">
                <button type="submit" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Proceed</button>
            </form>
            </body>
            </html>
            """, "text/html");
    }

    [HttpGet("device")]
    [ApiExplorerSettings(IgnoreApi = true)]
    public async Task<IActionResult> DeviceConfirm([FromQuery] string? user_code, [FromQuery] bool confirm)
    {
        ClearCookies();
        if (!confirm)
        {
            return RenderDeviceCodeUI(user_code);
        }

        var code = NormalizeUserCode(user_code);
        var deviceAuthz = await DbContext.DeviceAuthorization
            .FirstOrDefaultAsync(x => x.UserCode == code);
        if (deviceAuthz == null)
        {
            return RenderCallbackUI("Error", "Invalid code", "The code provided is not found in our records.", Url.Action(nameof(DeviceConfirm)));
        }
        if (deviceAuthz.UserId != null)
        {
            DbContext.Remove(deviceAuthz);
            await DbContext.SaveChangesAsync();
            return RenderCallbackUI("Error", "Invalid code", "The code provided has already been used.", Url.Action(nameof(DeviceConfirm)));
        }
        if (deviceAuthz.IsExpired)
        {
            DbContext.Remove(deviceAuthz);
            await DbContext.SaveChangesAsync();
            return RenderCallbackUI("Error", "Invalid code", "The code provided is expired.", Url.Action(nameof(DeviceConfirm)));
        }
        Response.Cookies.Append("user_code", code, new CookieOptions
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

    protected IActionResult RenderCallbackUI(string title, string message, string description, string? redirect = null)
    {
        return Content($"""
            <!doctype html>
            <html>
            <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <link href="/style.css" rel="stylesheet">
            </head>
            <body class="grid h-screen place-items-center bg-slate-100">
            <div class="container max-w-2xl bg-white shadow-2xl rounded-xl p-6 space-y-2">
                <h1 class="text-4xl font-bold">{title}</h1>
                <h2 class="text-2xl">{message}</h2>
                <p>{description}</p>
                <div>{(redirect != null ? $"""<a href="{redirect}" class="font-bold bg-sky-700 text-white px-2 py-1 rounded-md shadow-md hover:bg-sky-500">Retry</a>""" : "")}</div>
            </div>
            </body>
            </html>
            """, "text/html");
    }

    [HttpGet("callback/vatsim")]
    [ApiExplorerSettings(IgnoreApi = true)]
    public async Task<IActionResult> VatsimCallback(string? code, string? state, string? error)
    {
        if (string.IsNullOrEmpty(code))
        {
            if (string.IsNullOrEmpty(error))
            {
                return RenderCallbackUI("Error", "Missing code", "Are you coming from VATSIM Connect?");
            }
            else if (error == "access_denied")
            {
                return RenderCallbackUI("Error", "Access denied", "You have denied the request.", Url.Action(nameof(Login)));
            }
            else
            {
                Logger.LogError("VATSIM authn error: {error}", error);
                return RenderCallbackUI("Error", "Internal error", "Please try again later.", Url.Action(nameof(Login)));
            }
        }

        Logger.LogInformation("Recevied VATSIM authn callback");

        Request.Cookies.TryGetValue("code_verifier", out var verifier);
        Response.Cookies.Delete("code_verifier");
        Logger.LogInformation("Delete code_verifier cookie");

        VatsimAuthService.TokenResponse token;
        VatsimAuthService.UserResponse vatsimUser;
        try
        {
            token = await AuthService.GetTokenAsync(code, verifier ?? string.Empty);
            vatsimUser = await AuthService.GetUserAsync(token.AccessToken);
        }
        catch (FlurlHttpException e)
        {
            Logger.LogError(e, "Failed to get token or user info since {Response}", await e.GetResponseStringAsync());
            SentrySdk.CaptureException(e);
            return RenderCallbackUI("Error", "Internal error", "Please try again later.", Url.Action(nameof(Login)));
        }

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

            return RenderCallbackUI("Welcome", $"Hello {user.Cid}", "Login successful, please return to your device.");
        }

        return RenderCallbackUI("Welcome", $"Hello {user.Cid}", "You have been successfully registered in VATPRC's database. You may close this page.");
    }

    /// <summary>
    /// Access Token Request
    /// </summary>
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
        [ModelBinder(Name = "client_id")]
        public string ClientId { get; set; } = string.Empty;

        /// <summary>
        /// REQUIRED.  The device verification code, "device_code" from the
        /// device authorization response, defined in Section 3.2.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/rfc8628#section-3.5"/>
        [ModelBinder(Name = "device_code")]
        public string DeviceCode { get; set; } = string.Empty;

        /// <summary>
        /// REQUIRED. The refresh token issued to the client.
        /// </summary>
        /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-4.3.1"/>
        [ModelBinder(Name = "refresh_token")]
        public string RefreshToken { get; set; } = string.Empty;

        /// <summary>
        /// REQUIRED. The authorization code received from the authorization
        /// server.
        /// </summary>
        [ModelBinder(Name = "code")]
        public string Code { get; set; } = string.Empty;

        /// <summary>
        /// REQUIRED, if the code_challenge parameter was included in the
        /// authorization request. MUST NOT be used otherwise. The original
        /// code verifier string.
        /// </summary>
        [ModelBinder(Name = "code_verifier")]
        public string CodeVerifier { get; set; } = string.Empty;
    }

    /// <summary>
    /// Token response, compliant with RFC 8693
    /// </summary>
    /// <see href="https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-11#section-3.2.3"/>
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
        /// <example>Bearer</example>
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
    /// <returns></returns>
    [HttpPost("token")]
    [Consumes("application/x-www-form-urlencoded")]
    [ProducesResponseType(typeof(TokenResponse), StatusCodes.Status200OK)]
    [ProducesResponseType(typeof(TokenErrorDto), StatusCodes.Status400BadRequest)]
    public async Task<IActionResult> Token(
        [FromForm] AccessTokenRequest req)
    {
        if (req.GrantType == "urn:ietf:params:oauth:grant-type:device_code")
        {
            return await DeviceCodeGrant(req);
        }
        if (req.GrantType == "refresh_token")
        {
            return await RefreshTokenGrant(req);
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

    protected async Task<IActionResult> DeviceCodeGrant(AccessTokenRequest req)
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
        if (deviceAuthz.ClientId != req.ClientId)
        {
            Logger.LogInformation("Client ID mismatch: req {client_id} != db {device_client_id}",
                req.ClientId, deviceAuthz.ClientId);
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

    protected async Task<IActionResult> RefreshTokenGrant(AccessTokenRequest req)
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
