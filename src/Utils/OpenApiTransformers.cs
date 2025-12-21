#nullable disable

using System.Collections.Frozen;
using System.Text.Json.Nodes;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc.Controllers;
using Microsoft.AspNetCore.OpenApi;
using Microsoft.OpenApi;
using Net.Vatprc.Uniapi.Controllers.Auth;
using static Net.Vatprc.Uniapi.ApiError;

namespace Net.Vatprc.Uniapi.Utils;

public static class OpenApiTransformers
{
    public static Task TransformDocument(OpenApiDocument document, OpenApiDocumentTransformerContext context, CancellationToken ct)
    {
        document.Info.Title = "VATPRC UniAPI";
        document.Info.Version = "v1";
        document.Info.Description = """
        # Error Handling

        VATPRC UniAPI returns normalized error responses. The response body is a JSON object with the following fields:

        | Field           | Type     | Description     |
        | --------------- | -------- | --------------- |
        | `error_code`    | `string` | Error code.     |
        | `message`       | `string` | Error message.  |
        | `connection_id` | `string` | Connection ID.     |
        | `request_id`    | `string` | Request ID. |

        It may contain additional fields depending on the error code.

        For details, see the examples on each API endpoint. The additional fields is denoted like `{field}` in the
        error message example.
        """;
        document.Servers.Add(new OpenApiServer
        {
            Url = "https://uniapi.vatprc.net",
            Description = "Production server"
        });
        document.Components ??= new OpenApiComponents();
        document.Components.SecuritySchemes ??= new Dictionary<string, IOpenApiSecurityScheme>();
        document.Components.SecuritySchemes.Add("oauth2", new OpenApiSecurityScheme
        {
            Type = SecuritySchemeType.OAuth2,
            Flows = new OpenApiOAuthFlows
            {
                Password = new OpenApiOAuthFlow
                {
                    TokenUrl = new Uri("/api/session", UriKind.Relative),
                },
                AuthorizationCode = new OpenApiOAuthFlow
                {
                    AuthorizationUrl = new Uri("/auth/authorize", UriKind.Relative),
                    TokenUrl = new Uri("/auth/token", UriKind.Relative),
                    RefreshUrl = new Uri("/auth/token", UriKind.Relative),
                },
                ClientCredentials = new OpenApiOAuthFlow
                {
                    TokenUrl = new Uri("/auth/token", UriKind.Relative),
                },
            },
        });
        document.Security ??= new List<OpenApiSecurityRequirement>();
        document.Security.Add(new OpenApiSecurityRequirement
        {
            {
                new OpenApiSecuritySchemeReference("oauth2"), []
            }
        });
        return Task.CompletedTask;
    }

    public static Task AddUlid(OpenApiSchema schema, OpenApiSchemaTransformerContext context, CancellationToken ct)
    {
        if (context.JsonTypeInfo.Type == typeof(Ulid) && schema.Type == null)
        {
            schema.Type = JsonSchemaType.String;
            schema.Description = "ULID";
        }
        return Task.CompletedTask;
    }

    public static Task AllowAnonymous(OpenApiOperation operation, OpenApiOperationTransformerContext context, CancellationToken ct)
    {
        if (context.Description.ActionDescriptor is not ControllerActionDescriptor descriptor)
        {
            return Task.CompletedTask;
        }

        var allowAnonymous = descriptor.MethodInfo
            .GetCustomAttributes(true)
            .OfType<AllowAnonymousAttribute>()
            .Any() ||
            (descriptor.MethodInfo.DeclaringType
                ?.GetCustomAttributes(true)
                .OfType<AllowAnonymousAttribute>().Any()
            ?? true);
        if (allowAnonymous)
        {
            operation.Security = [[]];
        }

        return Task.CompletedTask;
    }

    public static Task AddErrorResponse(OpenApiOperation operation, OpenApiOperationTransformerContext context, CancellationToken ct)
    {
        if (context.Description.ActionDescriptor is not ControllerActionDescriptor descriptor)
        {
            return Task.CompletedTask;
        }
        if (descriptor.MethodInfo.DeclaringType == typeof(AuthController))
        {
            return Task.CompletedTask;
        }

        static ErrorAttribute GetErrorAttribute(Type type) =>
            (ErrorAttribute)type.GetCustomAttributes(typeof(ErrorAttribute), true).First();

        var exceptions = descriptor.MethodInfo
            .GetCustomAttributes(typeof(HasAttribute<>), true)
            .Select(x => ((IHasAttribute)x).ExceptionType)
            .Concat(new Type[]
            {
                typeof(InternalServerError),
                // if the security is empty, the operation uses the global policy (aka. required auth)
                operation.Security?.Count == 0 ? typeof(InvalidToken) : null!,
            }.Where(x => x != null))
            .Select(GetErrorAttribute)
            .GroupBy(x => x.StatusCode)
            .ToDictionary(x => x.Key, x => x.AsEnumerable());
        foreach (var exception in exceptions)
        {
            operation.Responses.Add(((int)exception.Key).ToString(), new OpenApiResponse
            {
                Description = string.Join(", ", exception.Value.Select(x => x.ErrorCode)),
                Content = new Dictionary<string, OpenApiMediaType>()
                {
                    ["application/json"] = new OpenApiMediaType
                    {
                        Schema = new OpenApiSchema
                        {
                            AnyOf = exception.Value.Select(x =>
                            {
                                var properties = new Dictionary<string, IOpenApiSchema>
                                {
                                    ["error_code"] = new OpenApiSchema
                                    {
                                        Type = JsonSchemaType.String,
                                        Enum = [JsonValue.Create(x.ErrorCode)],
                                        Description = x.ErrorCode
                                    },
                                    ["message"] = new OpenApiSchema
                                    {
                                        Type = JsonSchemaType.String,
                                        Example = JsonValue.Create(x.MessageExample),
                                        Description = x.MessageExample
                                    },
                                    ["type"] = new OpenApiSchema
                                    {
                                        Type = JsonSchemaType.String,
                                        Enum = [JsonValue.Create($"urn:vatprc-uniapi-error:{x.ErrorCode.ToLowerInvariant().Replace('_', '-')}")],
                                        Description = x.ErrorCode
                                    },
                                    ["title"] = new OpenApiSchema
                                    {
                                        Type = JsonSchemaType.String,
                                        Example = JsonValue.Create(x.MessageExample),
                                        Description = x.MessageExample
                                    },
                                    ["detail"] = new OpenApiSchema
                                    {
                                        Type = JsonSchemaType.String,
                                        Example = JsonValue.Create(x.MessageExample),
                                        Description = x.MessageExample
                                    },
                                };
                                return (IOpenApiSchema)new OpenApiSchema
                                {
                                    Type = JsonSchemaType.Object,
                                    Properties = properties,
                                    Required = properties.Keys.ToFrozenSet(),
                                    Description = "Error object compliant with RFC 7807",
                                };
                            }).ToList(),
                        },
                    },
                }
            });
        }

        return Task.CompletedTask;
    }
}
