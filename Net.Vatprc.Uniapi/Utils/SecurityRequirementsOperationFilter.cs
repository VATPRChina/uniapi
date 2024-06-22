using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.OpenApi.Any;
using Microsoft.OpenApi.Models;
using Swashbuckle.AspNetCore.SwaggerGen;

public class SecurityRequirementsOperationFilter : IOperationFilter
{
    public void Apply(OpenApiOperation operation, OperationFilterContext context)
    {
        var allowAnonymous = context.MethodInfo
            .GetCustomAttributes(true)
            .OfType<AllowAnonymousAttribute>()
            .Any() ||
            (context.MethodInfo.DeclaringType
                ?.GetCustomAttributes(true)
                .OfType<AllowAnonymousAttribute>().Any()
            ?? true);
        if (allowAnonymous)
        {
            operation.Security = new List<OpenApiSecurityRequirement>() { new() };
            // A workaround for https://github.com/acacode/swagger-typescript-api/pull/602
            operation.Extensions.Add("x-allow-anonymous", new OpenApiBoolean(true));
        }
        var controller = context.MethodInfo.DeclaringType?.GetCustomAttributes(true).OfType<RouteAttribute>().FirstOrDefault();
        operation.Extensions.Add("x-controller", new OpenApiObject
        {
            ["name"] = new OpenApiString(context.MethodInfo.DeclaringType?.Name ?? ""),
            ["route-template"] = new OpenApiString(controller?.Template ?? ""),
        });
        operation.Extensions.Add("x-action", new OpenApiObject
        {
            ["name"] = new OpenApiString(context.MethodInfo.Name),
        });
        operation.OperationId = $"{context.MethodInfo.DeclaringType?.Name.Replace("Controller", "")}_{context.MethodInfo.Name.Replace("Async", "")}";
    }
}
