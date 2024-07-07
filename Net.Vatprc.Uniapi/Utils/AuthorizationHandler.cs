using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Authorization.Infrastructure;
using Microsoft.AspNetCore.Authorization.Policy;

namespace Net.Vatprc.Uniapi.Utils;

public class AuthorizationHandler : IAuthorizationMiddlewareResultHandler
{
    protected readonly AuthorizationMiddlewareResultHandler DefaultHandler = new();

    public async Task HandleAsync(
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
            if (context.RequestServices.GetRequiredService<IHostEnvironment>().IsDevelopment())
            {
                await context.Response.WriteAsJsonAsync(new ApiError.ErrorDevResponse(err));
            }
            else
            {
                await context.Response.WriteAsJsonAsync(new ApiError.ErrorProdResponse(err));
            }
            return;
        }

        await DefaultHandler.HandleAsync(next, context, policy, authorizeResult);
    }
}
