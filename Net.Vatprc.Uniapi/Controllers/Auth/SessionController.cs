using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;

namespace Net.Vatprc.Uniapi.Controllers;

/// <summary>
/// Operate users.
/// </summary>
[ApiController, Route("api/session")]
public class SessionController(
    VATPRCContext DbContext
) : ControllerBase
{
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
