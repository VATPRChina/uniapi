using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Authorization.Infrastructure;
using Microsoft.AspNetCore.Authorization.Policy;

namespace Net.Vatprc.Uniapi.Utils;

public class AuthorizationHandler : IAuthorizationMiddlewareResultHandler
{
    protected readonly AuthorizationMiddlewareResultHandler DefaultHandler = new();

    public Task HandleAsync(
        RequestDelegate next,
        HttpContext context,
        AuthorizationPolicy policy,
        PolicyAuthorizationResult authorizeResult)
    {
        if (authorizeResult.Forbidden)
        {
            var allowedRoles = authorizeResult.AuthorizationFailure?.FailedRequirements
                .OfType<RolesAuthorizationRequirement>()
                .SelectMany(x => x.AllowedRoles) ?? [];
            context.Response.StatusCode = StatusCodes.Status403Forbidden;
            var err = new ApiError.Forbidden(allowedRoles);

            return context.Response.WriteAsJsonAsync(err.ToProblem(context));
        }

        return DefaultHandler.HandleAsync(next, context, policy, authorizeResult);
    }
}
