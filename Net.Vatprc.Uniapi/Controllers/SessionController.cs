using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Net.Vatprc.Uniapi.Services;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Authorization;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/session")]
public class SessionController : ControllerBase
{
    protected VATPRCContext DbContext { get; init; }
    protected TokenService TokenService { get; init; }

    public SessionController(VATPRCContext dbContext, TokenService tokenService)
    {
        DbContext = dbContext;
        TokenService = tokenService;
    }

    public record LoginReqDto
    {
        public string username { get; set; } = string.Empty;
        public string password { get; set; } = string.Empty;
        public string captcha { get; set; } = string.Empty;
        public string grant_type { get; set; } = string.Empty;
        public string refresh_token { get; set; } = string.Empty;
    }

    public record LoginResDto(
        string AccessToken,
        uint ExpiresIn,
        string RefreshToken,
        string Scope,
        string TokenType = "Bearer",
        string IssuedTokenType = "urn:ietf:params:oauth:token-type:access_token"
    );

    /// <summary>Login</summary>
    /// <remarks><![CDATA[
    /// Login with username and password. This API does not comply with OAuth 2.1,
    /// and only supports first-party applications (the built-in web frontend).
    /// It is based on `grant_type` `password` (which has been drooped in OAuth 2.1)
    /// or `refresh_token`. It requires additional parameters for security control.
    /// 
    /// **Request with password**
    /// 
    /// It requires `username`, `password`, `captcha`.
    /// 
    /// ```text
    /// username=alice&password=foobar&captcha=foobar&grant_type=password
    /// ```
    /// 
    /// **Request with refresh token**
    /// 
    /// It requires `refresh_token`.
    /// 
    /// ```text
    /// grant_type=refresh_token&refresh_token=507f0155-577e-448d-870b-5abe98a41d3f
    /// ```
    /// ]]></remarks>
    /// <param name="req"></param>
    /// <returns></returns>
    /// <exception cref="ApiError.InvalidGrantType">If the grant type is invalid.</exception>
    [HttpPost]
    [AllowAnonymous]
    [ResponseCache(NoStore = true)]
    [Consumes("application/x-www-form-urlencoded")]
    [ApiError.Has<ApiError.InvalidGrantType>]
    public async Task<LoginResDto> Login([FromForm] LoginReqDto req)
    {
        if (req.grant_type == "password")
        {
            var user = await DbContext.User.FirstOrDefaultAsync(x => x.Cid == req.username);
            // FIXME: password grant should only be used in dev
            if (user == null)
            {
                user = new()
                {
                    Cid = req.username,
                    FullName = req.username
                };
                DbContext.User.Add(user);
                await DbContext.SaveChangesAsync();
            }
            var refresh = await TokenService.IssueFirstPartyRefreshToken(user, null);
            var (token, jwt) = TokenService.IssueFirstParty(user, refresh);
            var expires = jwt.Payload.Expiration ?? 0;
            var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            var scopes = jwt.Payload.Claims.FirstOrDefault(x => x.Type == TokenService.JwtClaimNames.Scope)?.Value ?? "";
            return new(
                AccessToken: token,
                ExpiresIn: (uint)(expires - now),
                RefreshToken: refresh.Token.ToString(),
                Scope: scopes);
        }
        else if (req.grant_type == "refresh_token")
        {
            if (!Ulid.TryParse(req.refresh_token, out var tokenId))
            {
                throw new ApiError.InvalidRefreshToken("not_guid");
            }
            var refresh = await DbContext.Session
                .Include(x => x.User)
                .FirstOrDefaultAsync(x => x.Token == tokenId) ??
                throw new ApiError.InvalidRefreshToken("not_found");
            if (refresh.UserUpdatedAt != refresh.User.UpdatedAt)
            {
                throw new ApiError.InvalidRefreshToken("user_updated");
            }
            if (refresh.ExpiresIn < DateTimeOffset.Now)
            {
                throw new ApiError.InvalidRefreshToken("token_expired");
            }
            var newRefresh = await TokenService.IssueFirstPartyRefreshToken(refresh.User, refresh);
            var (token, jwt) = TokenService.IssueFirstParty(refresh.User, newRefresh);
            var expires = jwt.Payload.Expiration ?? 0;
            var now = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            var scopes = jwt.Payload.Claims.FirstOrDefault(x => x.Type == TokenService.JwtClaimNames.Scope)?.Value ?? "";
            return new(
                AccessToken: token,
                ExpiresIn: (uint)(expires - now),
                RefreshToken: newRefresh.Token.ToString(),
                Scope: scopes);
        }
        else
        {
            throw new ApiError.InvalidGrantType(req.grant_type);
        }
    }

    public record TokenDto(
        UserController.UserDto User,
        DateTimeOffset IssuedAt,
        DateTimeOffset ExpiresAt
    );

    /// <summary>Get Current</summary>
    /// <returns></returns>
    /// <exception cref="ApiError.InvalidTokenNotFirstParty"></exception>
    [HttpGet]
    [ApiError.Has<ApiError.InvalidTokenNotFirstParty>]
    public async Task<TokenDto> Get()
    {
        if (!TokenService.IsFirstParty(User))
        {
            throw new ApiError.InvalidTokenNotFirstParty();
        }
        var expires = User.FindFirstValue(JwtRegisteredClaimNames.Exp);
        var issued = User.FindFirstValue(JwtRegisteredClaimNames.Iat);
        var subject = User.FindFirstValue(ClaimTypes.NameIdentifier);
        var userId = Ulid.Parse(subject);
        var user = await DbContext.User.FindAsync(userId);
        return new(
            new(user!),
            DateTimeOffset.FromUnixTimeSeconds(Convert.ToInt64(issued)),
            DateTimeOffset.FromUnixTimeSeconds(Convert.ToInt64(expires))
        );
    }

    /// <summary>Logout</summary>
    /// <returns></returns>
    /// <exception cref="ApiError.InvalidTokenNotFirstParty"></exception>
    /// <exception cref="ApiError.InvalidToken"></exception>
    [HttpDelete]
    [ProducesResponseType(StatusCodes.Status204NoContent)]
    [ApiError.Has<ApiError.InvalidTokenNotFirstParty>]
    public async Task<IActionResult> Logout()
    {
        if (!TokenService.IsFirstParty(User))
        {
            throw new ApiError.InvalidTokenNotFirstParty();
        }
        var refresh = User.FindFirstValue(JwtRegisteredClaimNames.Sid) ??
            throw new ApiError.InvalidToken("vatprc_sid_not_present", "no sid in token", null);
        var tokenId = Guid.Parse(refresh);
        var token = await DbContext.Session.FindAsync(tokenId) ??
            throw new ApiError.InvalidToken("vatprc_refresh_token_not_found", $"refresh token {refresh} not found", null);
        DbContext.Session.Remove(token);
        await DbContext.SaveChangesAsync();
        return NoContent();
    }
}
