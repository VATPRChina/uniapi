using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Dto;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Controllers.Auth;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/session")]
public class SessionController(
    DatabaseAdapter DbContext
) : ControllerBase
{
    /// <summary>Get Current</summary>
    /// <returns></returns>
    [HttpGet]
    public async Task<TokenDto> Get()
    {
        var expires = User.FindFirstValue(JwtRegisteredClaimNames.Exp);
        var issued = User.FindFirstValue(JwtRegisteredClaimNames.Iat);
        var subject = User.FindFirstValue(ClaimTypes.NameIdentifier);
        var userId = Ulid.Parse(subject);
        var user = await DbContext.User.FindAsync(userId);
        var identity = HttpContext.User.Identity as ClaimsIdentity;
        var roles = identity?.FindAll(ClaimTypes.Role).Select(x => x.Value).ToHashSet();
        return new()
        {
            User = UserDto.From(user!, showFullName: true, roles: roles),
            IssuedAt = DateTimeOffset.FromUnixTimeSeconds(Convert.ToInt64(issued)),
            ExpiresAt = DateTimeOffset.FromUnixTimeSeconds(Convert.ToInt64(expires)),
        };
    }

    /// <summary>Logout</summary>
    /// <returns></returns>
    /// <exception cref="ApiError.InvalidToken"></exception>
    [HttpDelete]
    [ProducesResponseType(StatusCodes.Status204NoContent)]
    public async Task<IActionResult> Logout()
    {
        var refresh = User.FindFirstValue(JwtRegisteredClaimNames.Sid) ??
            throw new ApiError.InvalidToken("vatprc_sid_not_present", "no sid in token", null);
        var tokenId = Ulid.Parse(refresh);
        var token = await DbContext.Session.FindAsync(tokenId) ??
            throw new ApiError.InvalidToken("vatprc_refresh_token_not_found", $"refresh token {refresh} not found", null);
        DbContext.Session.Remove(token);
        await DbContext.SaveChangesAsync();
        return NoContent();
    }
}
