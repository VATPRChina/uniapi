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
    VatsimAuthService AuthService,
    VATPRCContext DbContext) : ControllerBase
{
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
            .SetQueryParam("code_challenge_method", "S256");
        Response.Cookies.Append("code_verifier", verifier, new CookieOptions
        {
            HttpOnly = true,
            Secure = true,
            SameSite = SameSiteMode.Strict,
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
                FullName = vatsimUser.Data.Cid
            };
            DbContext.User.Add(user);
            await DbContext.SaveChangesAsync();
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
