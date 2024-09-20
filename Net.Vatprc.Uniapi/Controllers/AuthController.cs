using Flurl;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers;

[ApiController, Route("auth")]
[AllowAnonymous]
public class AuthController(
    IOptions<VatsimAuthService.Option> Options,
    TokenService TokenService,
    VatsimAuthService AuthService,
    VATPRCContext DbContext) : ControllerBase
{
    [HttpGet("authorize")]
    public IActionResult Authorize(
        [FromQuery] string response_type,
        [FromQuery] string client_id,
        [FromQuery] string redirect_uri)
    {
        if (response_type != "code")
        {
            return BadRequest("invalid response_type");
        }
        if (!TokenService.CheckClientExists(client_id, redirect_uri))
        {
            return Unauthorized("client is invalid");
        }
        Response.Cookies.Append("client_id", client_id, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
        });
        Response.Cookies.Append("redirect", redirect_uri, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Lax,
        });
        return RedirectToActionPreserveMethod(nameof(Login), "Auth");
    }

    [HttpGet("login")]
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
        });
        return RedirectPreserveMethod(url.ToString());
    }

    [HttpGet("callback/vatsim")]
    public async Task<IActionResult> VatsimCallback(string code, string? state)
    {
        Request.Cookies.TryGetValue("code_verifier", out var verifier);
        Response.Cookies.Delete("code_verifier");
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
            Response.Cookies.Delete("client_id");
            Response.Cookies.Delete("redirect");
            var refresh = await TokenService.IssueFirstPartyRefreshToken(user, createCode: true);
            var authCode = TokenService.GenerateAuthCode(refresh, clientId, redirect.ToString());
            var redirectTarget = redirectUri
                .SetQueryParam("code", authCode)
                .ToString();
            return RedirectPreserveMethod(redirectTarget);
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
}
