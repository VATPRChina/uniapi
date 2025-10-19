using System.Security.Claims;
using System.Text;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.Net.Http.Headers;
using Net.Vatprc.Uniapi.Models;
using Net.Vatprc.Uniapi.Services;

namespace Net.Vatprc.Uniapi.Utils;

public class AuthenticationEventHandler(VATPRCContext DbContext) : JwtBearerEvents
{
    public override async Task Challenge(JwtBearerChallengeContext context)
    {
        context.HandleResponse();

        ApiError err = context.AuthenticateFailure switch
        {
            ApiError e => e,
            null or Exception => new ApiError.InvalidToken(
                context.Error,
                context.ErrorDescription,
                context.AuthenticateFailure
            ),
        };

        // From: https://github.com/dotnet/aspnetcore/blob/9402bfac90a695bb732dff17ba624801076df77f/src/Security/Authentication/JwtBearer/src/JwtBearerHandler.cs#L296-L345
        #region original logic
        context.Response.StatusCode = 401;

        if (string.IsNullOrEmpty(context.Error) &&
            string.IsNullOrEmpty(context.ErrorDescription) &&
            string.IsNullOrEmpty(context.ErrorUri))
        {
            context.Response.Headers.Append(HeaderNames.WWWAuthenticate, context.Options.Challenge);
        }
        else
        {
            // https://tools.ietf.org/html/rfc6750#section-3.1
            // WWW-Authenticate: Bearer realm="example", error="invalid_token", error_description="The access token expired"
            var builder = new StringBuilder(context.Options.Challenge);
            if (context.Options.Challenge.IndexOf(' ') > 0)
            {
                // Only add a comma after the first param, if any
                builder.Append(',');
            }
            if (!string.IsNullOrEmpty(context.Error))
            {
                builder.Append(" error=\"");
                builder.Append(context.Error);
                builder.Append('\"');
            }
            if (!string.IsNullOrEmpty(context.ErrorDescription))
            {
                if (!string.IsNullOrEmpty(context.Error))
                {
                    builder.Append(',');
                }

                builder.Append(" error_description=\"");
                builder.Append(context.ErrorDescription);
                builder.Append('\"');
            }
            if (!string.IsNullOrEmpty(context.ErrorUri))
            {
                if (!string.IsNullOrEmpty(context.Error) ||
                    !string.IsNullOrEmpty(context.ErrorDescription))
                {
                    builder.Append(',');
                }

                builder.Append(" error_uri=\"");
                builder.Append(context.ErrorUri);
                builder.Append('\"');
            }

            context.Response.Headers.Append(HeaderNames.WWWAuthenticate, builder.ToString());
        }
        #endregion

        // Customized logic for JSON response
        await context.HttpContext.Response.WriteAsJsonAsync(err.ToProblem(context.HttpContext));
    }

    protected Task Validate(TokenValidatedContext context)
    {
        if (context.Principal == null)
        {
            throw new ApiError.InvalidToken("vatprc_no_principal", "no principal in token", null);
        }
        var subject = context.Principal.FindFirstValue(ClaimTypes.NameIdentifier) ??
            throw new ApiError.InvalidToken("vatprc_no_subject", "no subject in token", null);
        if (Ulid.TryParse(subject, out var _) == false)
        {
            throw new ApiError.InvalidToken("vatprc_invalid_subject", "subject is not ulid", null);
        }
        return Task.CompletedTask;
    }

    public override async Task TokenValidated(TokenValidatedContext context)
    {
        try
        {
            await Validate(context);
            if (context.Principal == null) return;
            var id = Ulid.Parse(context.Principal.FindFirstValue(ClaimTypes.NameIdentifier));
            var user = await DbContext.User.FindAsync(id);
            var identity = (ClaimsIdentity)context.Principal.Identity!;
            if (identity == null) return;
            if (user == null)
            {
                identity.AddClaim(new(ClaimTypes.Role, User.SpecialRoles.ApiClient));
                return;
            }
            identity.AddClaim(new(ClaimTypes.Role, User.SpecialRoles.User));
            var allRoles = UserRoleService.GetRoleClosure(new HashSet<string>(user.Roles));
            foreach (var role in allRoles)
            {
                if (!identity.HasClaim(ClaimTypes.Role, role))
                {
                    identity.AddClaim(new(ClaimTypes.Role, role));
                }
            }
        }
        catch (Exception e)
        {
            context.Fail(e);
        }
    }

    public override Task AuthenticationFailed(AuthenticationFailedContext context)
    {
        context.HttpContext.RequestServices.GetRequiredService<ILogger<AuthenticationEventHandler>>()
            .LogError(context.Exception, "Authentication failed");
        return Task.CompletedTask;
    }

    public override Task Forbidden(ForbiddenContext context)
    {
        context.HttpContext.RequestServices.GetRequiredService<ILogger<AuthenticationEventHandler>>()
            .LogError("Forbidden");
        return Task.CompletedTask;
    }
}
