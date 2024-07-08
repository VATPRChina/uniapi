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
    VatsimAuthService AuthService) : ControllerBase
{
    [HttpGet("login")]
    public IActionResult Login()
    {
        var url = new Url(Options.Value.Endpoint)
            .AppendPathSegment("oauth/authorize")
            .SetQueryParam("response_type", "code")
            .SetQueryParam("client_id", Options.Value.ClientId)
            .SetQueryParam("redirect_uri", Options.Value.RedirectUri)
            .SetQueryParam("state", "foobar")
            .SetQueryParam("code_challenge", "LIknhjPJT-eMAZqhQMOgu3CcPNAXXorpXSPx6DkBh8g")
            .SetQueryParam("code_challenge_method", "S256");
        return RedirectPreserveMethod(url.ToString());
    }

    [HttpGet("callback/vatsim")]
    public async Task<IActionResult> VatsimCallback(string code, string state)
    {
        var token = await AuthService.GetTokenAsync(code);
        var user = await AuthService.GetUserAsync(token.AccessToken);
        return Ok(new
        {
            User = user,
            Code = code,
            Token = token,
            State = state,
        });
    }
}
