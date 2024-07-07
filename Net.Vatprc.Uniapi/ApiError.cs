using System.Net;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;
using Microsoft.AspNetCore.Mvc.ModelBinding;
using Microsoft.OpenApi.Models;
using Swashbuckle.AspNetCore.SwaggerGen;

namespace Net.Vatprc.Uniapi;

public abstract class ApiError : Exception
{
    public HttpStatusCode StatusCode { get; set; }

    public string ErrorCode { get; set; }

    public IDictionary<string, object> ExtraData { get; set; }

    public ApiError(
        HttpStatusCode statusCode,
        string errorCode,
        string message,
        Exception? innerException = null) : this(
            statusCode,
            errorCode,
            message,
            new Dictionary<string, object>(),
            innerException)
    {
    }

    public ApiError(
        HttpStatusCode statusCode,
        string errorCode,
        string message,
        IDictionary<string, object> extraData,
        Exception? innerException = null) : base(message, innerException)
    {
        StatusCode = statusCode;
        ErrorCode = errorCode;
        ExtraData = extraData;
    }

    protected ApiError(
        string message,
        Exception? innerException = null) : this(
            message,
            new Dictionary<string, object>(),
            innerException)
    {
    }

    protected ApiError(
        string message,
            IDictionary<string, object> extraData,
        Exception? innerException = null) : base(message, innerException)
    {
        var type = GetType().GetCustomAttributes(typeof(ErrorAttribute), true).FirstOrDefault() as ErrorAttribute ??
            throw new Exception("ErrorAttribute not found, cannot use this constructor.");
        StatusCode = type.StatusCode;
        ErrorCode = type.ErrorCode;
        ExtraData = extraData;
    }

    [AttributeUsage(AttributeTargets.Class)]
    protected class ErrorAttribute(HttpStatusCode statusCode, string errorCode, string messageExample = "") : Attribute
    {
        public HttpStatusCode StatusCode { get; set; } = statusCode;
        public string ErrorCode { get; set; } = errorCode;
        public string MessageExample { get; set; } = messageExample;
    }

    [AttributeUsage(AttributeTargets.Class, AllowMultiple = true)]
    protected class WithExtraData(string fieldName, Type fieldType) : Attribute
    {
        public string FieldName { get; set; } = fieldName;
        public Type FieldType { get; set; } = fieldType;
    }

    public void WithHttpContext(HttpContext context)
    {
        ExtraData.Add("connection_id", context.Connection.Id);
        ExtraData.Add("request_id", context.TraceIdentifier);
    }

    public record ErrorProdResponse(
        [property:JsonPropertyName("error_code")]
        string ErrorCode,
        [property:JsonPropertyName("message")]
        string Message,
        [property:JsonExtensionData]
        IDictionary<string, object> ExtraData
    )
    {
        public ErrorProdResponse(ApiError e) : this(
            e.ErrorCode,
            e.Message,
            e.ExtraData)
        {
        }

    };

    public record ErrorDevResponse(
        [property:JsonPropertyName("error_code")]
        string ErrorCode,
        [property:JsonPropertyName("message")]
        string Message,
        [property:JsonPropertyName("stack_trace")]
        string StackTrace,
        [property:JsonExtensionData]
        IDictionary<string, object> ExtraData
    )
    {
        public ErrorDevResponse(ApiError e) : this(
              e.ErrorCode,
              e.Message,
              e.StackTrace ?? e.InnerException?.StackTrace ?? "No stacktrace provided.",
              e.ExtraData)
        {
        }
    };

    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public class HasAttribute<T> : Attribute, IHasAttribute where T : ApiError
    {
        public Type ExceptionType { get; set; }
        public HasAttribute() =>
            ExceptionType = typeof(T);
    }

    protected interface IHasAttribute
    {
        public Type ExceptionType { get; }
    }

    public class ErrorResponsesOperationFilter : IOperationFilter
    {
        private ErrorAttribute GetErrorAttribute(Type type) =>
            (ErrorAttribute)type.GetCustomAttributes(typeof(ErrorAttribute), true).FirstOrDefault()!;

        public void Apply(OpenApiOperation operation, OperationFilterContext context)
        {
            var exceptions = context.MethodInfo
                .GetCustomAttributes(typeof(HasAttribute<>), true)
                .Select(x => ((IHasAttribute)x).ExceptionType)
                .Select(GetErrorAttribute)
                .Concat((IEnumerable<ErrorAttribute>)new[]
                {
                    GetErrorAttribute(typeof(InternalServerError)),
                    // if the security is empty, the operation uses the global policy (aka. required auth)
                    operation.Security.Count == 0 ? GetErrorAttribute(typeof(InvalidToken)) : null,
                }.Where(x => x != null))
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
                            Schema = context.SchemaGenerator.GenerateSchema(
                                typeof(ErrorProdResponse), context.SchemaRepository),
                            Examples = exception.Value.ToDictionary(x => x.ErrorCode, x => new OpenApiExample
                            {
                                Value = OpenApiAnyFactory.CreateFromJson(JsonSerializer.Serialize(new
                                {
                                    error_code = x.ErrorCode,
                                    message = x.MessageExample,
                                })),
                            })
                        },
                    }
                });
            }
        }
    }

    public class ErrorExceptionFilter(IHostEnvironment HostEnvironment, Serilog.ILogger Logger) : IExceptionFilter
    {
        private static IActionResult NormalizeError(Exception exception, ExceptionContext context, bool isProduction)
        {
            var error = exception switch
            {
                ApiError e => e,
                Exception e => new InternalServerError(e),
                null => new InternalServerError(new Exception("Null exception thrown.")),
            };

            error.WithHttpContext(context.HttpContext);
            error.ExtraData.Add("action_id", context.ActionDescriptor.Id);

            if (isProduction)
            {
                return new ObjectResult(new ErrorProdResponse(error))
                { StatusCode = (int)error.StatusCode };
            }
            else
            {
                return new ObjectResult(new ErrorDevResponse(error))
                { StatusCode = (int)error.StatusCode };
            }
        }

        public void OnException(ExceptionContext context)
        {
            if (context.Exception is Exception e and not ApiError)
            {
                Logger.Error(e, "Internal error occurred.");
            }

            context.Result = NormalizeError(
                context.Exception,
                context,
                HostEnvironment.IsProduction());
        }
    }

    [Error(HttpStatusCode.InternalServerError, "INTERNAL_SERVER_ERROR", "An internal server error occurred.")]
    public class InternalServerError(Exception innerException) : ApiError(
        $"An internal server error occurred: {innerException.Message}",
        innerException)
    {
    }

    [Error(HttpStatusCode.NotFound, "ENDPOINT_NOT_FOUND", "API endpoint not found.")]
    public class EndpointNotFound : ApiError
    {
        public EndpointNotFound() : base("API endpoint not found.") { }
    }

    [Error(HttpStatusCode.BadRequest, "BAD_REQUEST", "Request body is invalid.")]
    [WithExtraData("errors", typeof(IDictionary<string, IEnumerable<string>>))]
    public class BadRequest : ApiError
    {
        public BadRequest(ModelStateDictionary state) : base(
            "Request body is invalid.")
        {
            ExtraData.Add("errors", state
                .Select(e => new { e.Key, Value = e.Value?.Errors.Select(e => e.ErrorMessage) })
                .ToDictionary(e => e.Key, e => e.Value));
        }

        public BadRequest(string message) : base(
            $"Request body is invalid for {message}.")
        {
        }
    }

    [Error(HttpStatusCode.NotFound, "USER_NOT_FOUND", "User {user_id} not found.")]
    [WithExtraData("user_id", typeof(string))]
    public class UserNotFound : ApiError
    {
        public UserNotFound(Ulid id) : base(
            $"User {id} not found.")
        {
            ExtraData.Add("user_id", id);
        }
    }

    [Error(HttpStatusCode.BadRequest, "INVALID_GRANT_TYPE", "Invalid grant type {grant_type}.")]
    [WithExtraData("grant_type", typeof(string))]
    public class InvalidGrantType : ApiError
    {
        public InvalidGrantType(string grantType) : base(
            $"Invalid grant type {grantType}.")
        {
            ExtraData.Add("grant_type", grantType);
        }
    }

    [Error(HttpStatusCode.Unauthorized, "INVALID_TOKEN", "Invalid token {oauth_code}: {oauth_desc}.")]
    [WithExtraData("oauth_code", typeof(string))]
    [WithExtraData("oauth_desc", typeof(string))]
    public class InvalidToken : ApiError
    {
        public InvalidToken(string? code, string? description, Exception? ex) : base(
            $"Invalid token {code}: {description}.",
            ex)
        {
            ExtraData.Add("oauth_code", code ?? string.Empty);
            ExtraData.Add("oauth_desc", description ?? string.Empty);
        }
    }

    [Error(HttpStatusCode.Forbidden, "INVALID_TOKEN_NOT_FIRST_PARTY", "Token is not issued to first-party application.")]
    public class InvalidTokenNotFirstParty : ApiError
    {
        public InvalidTokenNotFirstParty() : base(
            $"Token is not issued to first-party application.")
        {
        }
    }

    [Error(HttpStatusCode.Forbidden, "INVALID_REFRESH_TOKEN", "Refresh token is not valid for {code}.")]
    [WithExtraData("code", typeof(string))]
    public class InvalidRefreshToken : ApiError
    {
        public InvalidRefreshToken(string code) : base(
            $"Refresh token is not valid for {code}.")
        {
            ExtraData.Add("code", code);
        }
    }

    [Error(HttpStatusCode.NotFound, "EVENT_NOT_FOUND", "Event {event_id} not found.")]
    [WithExtraData("event_id", typeof(string))]
    public class EventNotFound : ApiError
    {
        public EventNotFound(Ulid id) : base(
            $"Event {id} not found.")
        {
            ExtraData.Add("event_id", id);
        }
    }

    [Error(HttpStatusCode.NotFound, "EVENT_AIRSPACE_NOT_FOUND", "Event {event_id}'s airspace {airspace_id} not found.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("airspace_id", typeof(string))]
    public class EventAirspaceNotFound : ApiError
    {
        public EventAirspaceNotFound(Ulid event_id, Ulid airspace_id) : base(
            $"Event {event_id}'s airspace {airspace_id} not found.")
        {
            ExtraData.Add("event_id", event_id);
            ExtraData.Add("airspace_id", airspace_id);
        }
    }

    [Error(HttpStatusCode.NotFound, "EVENT_SLOT_NOT_FOUND", "Event {event_id}'s slot {slot_id} not found.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("slot_id", typeof(string))]
    public class EventSlotNotFound : ApiError
    {
        public EventSlotNotFound(Ulid event_id, Ulid slot_id) : base(
            $"Event {event_id}'s slot {slot_id} not found.")
        {
            ExtraData.Add("event_id", event_id);
            ExtraData.Add("slot_id", slot_id);
        }
    }

    [Error(HttpStatusCode.NotFound, "EVENT_SLOT_NOT_BOOKED", "Event {event_id}'s slot {slot_id} has not been booked.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("slot_id", typeof(string))]
    public class EventSlotNotBooked : ApiError
    {
        public EventSlotNotBooked(Ulid event_id, Ulid slot_id) : base(
            $"Event {event_id}'s slot {slot_id} has not been booked.")
        {
            ExtraData.Add("event_id", event_id);
            ExtraData.Add("slot_id", slot_id);
        }
    }

    [Error(HttpStatusCode.Conflict, "EVENT_SLOT_BOOKED", "Event {event_id}'s slot {slot_id} has been booked.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("slot_id", typeof(string))]
    public class EventSlotBooked : ApiError
    {
        public EventSlotBooked(Ulid event_id, Ulid slot_id) : base(
            $"Event {event_id}'s slot {slot_id} has been booked.")
        {
            ExtraData.Add("event_id", event_id);
            ExtraData.Add("slot_id", slot_id);
        }
    }

    [Error(HttpStatusCode.Conflict, "EVENT_BOOK_MAXIMUM_EXCEEDED", "Event {event_id}'s booking limit for current user has been exceeded.")]
    [WithExtraData("event_id", typeof(string))]
    public class EventBookMaximumExceeded : ApiError
    {
        public EventBookMaximumExceeded(Ulid event_id) : base(
            $"Event {event_id}'s booking limit for current user has been exceeded.")
        {
            ExtraData.Add("event_id", event_id);
        }
    }

    [Error(HttpStatusCode.Forbidden, "EVENT_SLOT_BOOKED_BY_ANOTHER_USER", "Event {event_id}'s slot {slot_id} has been booked by another user.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("slot_id", typeof(string))]
    public class EventSlotBookedByAnotherUser : ApiError
    {
        public EventSlotBookedByAnotherUser(Ulid event_id, Ulid slot_id) : base(
            $"Event {event_id}'s slot {slot_id} has been booked by another user.")
        {
            ExtraData.Add("event_id", event_id);
            ExtraData.Add("slot_id", slot_id);
        }
    }

    [Error(HttpStatusCode.Forbidden, "EVENT_NOT_IN_BOOKING_TIME", "Event {event_id} is not in booking time.")]
    [WithExtraData("event_id", typeof(string))]
    [WithExtraData("slot_id", typeof(string))]
    public class EventNotInBookingTime : ApiError
    {
        public EventNotInBookingTime(Ulid event_id) : base(
            $"Event {event_id} is not in booking time.")
        {
            ExtraData.Add("event_id", event_id);
        }
    }

    [Error(HttpStatusCode.Forbidden, "FORBIDDEN", "Permission is not sufficient, lacks {roles}.")]
    [WithExtraData("required_roles", typeof(IEnumerable<string>))]
    public class Forbidden : ApiError
    {
        public Forbidden(IEnumerable<string> required_roles) : base(
            $"Permission is not sufficient, lacks {string.Join(",", required_roles)}.")
        {
            ExtraData.Add("required_roles", required_roles);
        }
    }
}
