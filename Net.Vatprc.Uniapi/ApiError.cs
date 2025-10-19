using System.Diagnostics;
using System.Net;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Mvc.Filters;
using Microsoft.AspNetCore.Mvc.ModelBinding;

namespace Net.Vatprc.Uniapi;

public abstract class ApiError : Exception
{
    public HttpStatusCode StatusCode { get; set; }

    public string ErrorCode { get; set; }

    public ApiError(
        HttpStatusCode statusCode,
        string errorCode,
        string message,
        Exception? innerException = null) : base(message, innerException)
    {
        StatusCode = statusCode;
        ErrorCode = errorCode;
    }

    protected ApiError(
        string message,
        Exception? innerException = null) : base(message, innerException)
    {
        var type = GetType().GetCustomAttributes(typeof(ErrorAttribute), true).FirstOrDefault() as ErrorAttribute ??
            throw new Exception("ErrorAttribute not found, cannot use this constructor.");
        StatusCode = type.StatusCode;
        ErrorCode = type.ErrorCode;
    }

    public ProblemDetails ToProblem(HttpContext context)
    {
        var problem = new ProblemDetails
        {
            Type = $"urn:vatprc-uniapi-error:{ErrorCode.ToLowerInvariant().Replace('_', '-')}",
            Title = Message,
            Status = (int)StatusCode,
            Detail = Message
        };

        // Gets or sets the current operation (Activity) for the current thread.
        if (Activity.Current?.Id != null)
            problem.Extensions.Add("trace_id", Activity.Current?.Id!);
        if (Activity.Current?.ParentId != null)
            problem.Extensions.Add("trace_parent_id", Activity.Current?.ParentId!);
        problem.Extensions.Add("connection_id", context.Connection.Id);
        // Gets or sets a unique identifier to represent this request in trace logs.
        problem.Extensions.Add("request_id", context.TraceIdentifier);

        // For backward compat
        problem.Extensions.Add("error_code", ErrorCode);
        problem.Extensions.Add("message", Message);

        return problem;
    }

    [AttributeUsage(AttributeTargets.Class)]
    internal class ErrorAttribute(HttpStatusCode statusCode, string errorCode, string messageExample = "") : Attribute
    {
        public HttpStatusCode StatusCode { get; set; } = statusCode;
        public string ErrorCode { get; set; } = errorCode;
        public string MessageExample { get; set; } = messageExample;
    }

    [AttributeUsage(AttributeTargets.Method, AllowMultiple = true)]
    public class HasAttribute<T> : Attribute, IHasAttribute where T : ApiError
    {
        public Type ExceptionType { get; set; }
        public HasAttribute() =>
            ExceptionType = typeof(T);
    }

    internal interface IHasAttribute
    {
        public Type ExceptionType { get; }
    }

    public class ErrorExceptionFilter(
        IHostEnvironment HostEnvironment,
        Serilog.ILogger Logger) : IExceptionFilter
    {
        public void OnException(ExceptionContext context)
        {
            var exception = context.Exception;
            if (exception is not ApiError)
            {
                Logger.Error(exception, "Internal error occurred.");
                exception.SetSentryMechanism(nameof(ErrorExceptionFilter), handled: false);
                SentrySdk.CaptureException(exception);
            }

            var error = exception switch
            {
                ApiError e => e,
                Exception e => new InternalServerError(e),
                null => new InternalServerError(new Exception("Null exception thrown.")),
            };

            var problem = error.ToProblem(context.HttpContext);
            if (HostEnvironment.IsDevelopment())
            {
                problem.Extensions.Add("stack_trace", error.StackTrace);
            }

            context.Result = new JsonResult(problem)
            {
                StatusCode = problem.Status,
                ContentType = "application/problem+json",
            };
        }
    }

    // FIXME: This is not usable until https://github.com/dotnet/aspnetcore/pull/59074.
    // public class ErrorExceptionHandler(IHostEnvironment HostEnvironment, Serilog.ILogger Logger) : IExceptionHandler
    // {
    //     public async ValueTask<bool> TryHandleAsync(HttpContext httpContext, Exception exception, CancellationToken cancellationToken)
    //     {
    //         if (exception is not ApiError)
    //         {
    //             Logger.Error(exception, "Internal error occurred.");
    //             exception.SetSentryMechanism(nameof(ErrorExceptionHandler), "Description", handled: false);
    //             SentrySdk.CaptureException(exception);
    //         }
    // 
    //         var error = exception switch
    //         {
    //             ApiError e => e,
    //             Exception e => new InternalServerError(e),
    //             null => new InternalServerError(new Exception("Null exception thrown.")),
    //         };
    // 
    //         var problem = new ProblemDetails
    //         {
    //             Type = $"urn:vatprc-uniapi-error:{error.ErrorCode.ToLowerInvariant().Replace('_', '-')}",
    //             Title = error.Message,
    //             Status = (int)error.StatusCode,
    //             Detail = error.Message
    //         };
    // 
    //         // Gets or sets the current operation (Activity) for the current thread.
    //         if (Activity.Current?.Id != null)
    //             problem.Extensions.Add("trace_id", Activity.Current?.Id!);
    //         if (Activity.Current?.ParentId != null)
    //             problem.Extensions.Add("trace_parent_id", Activity.Current?.ParentId!);
    //         problem.Extensions.Add("connection_id", httpContext.Connection.Id);
    //         // Gets or sets a unique identifier to represent this request in trace logs.
    //         problem.Extensions.Add("request_id", httpContext.TraceIdentifier);
    // 
    //         foreach (var field in error.GetType().GetFields())
    //         {
    //             problem.Extensions.Add(field.Name, field.GetValue(exception));
    //         }
    // 
    //         // For backward compat
    //         problem.Extensions.Add("error_code", error.ErrorCode);
    //         problem.Extensions.Add("message", error.Message);
    // 
    //         if (HostEnvironment.IsDevelopment())
    //         {
    //             problem.Extensions.Add("stack_trace", error.StackTrace);
    //         }
    // 
    //         await httpContext.Response.WriteAsJsonAsync(problem, cancellationToken);
    // 
    //         return true;
    //     }
    // }

    [Error(HttpStatusCode.InternalServerError, "INTERNAL_SERVER_ERROR", "An internal server error occurred.")]
    public class InternalServerError(Exception innerException) :
        ApiError($"An internal server error occurred: {innerException.Message}", innerException);

    [Error(HttpStatusCode.NotFound, "ENDPOINT_NOT_FOUND", "API endpoint not found.")]
    public class EndpointNotFound : ApiError
    {
        public EndpointNotFound() : base("API endpoint not found.") { }
    }

    [Error(HttpStatusCode.BadRequest, "BAD_REQUEST", "Request body is invalid.")]
    public class BadRequest : ApiError
    {
        IDictionary<string, IEnumerable<string>>? Errors { get; set; } = null;

        public BadRequest(ModelStateDictionary state) : base(
            "Request body is invalid.")
        {
            Errors = state
                .Select(e => new { e.Key, Value = e.Value?.Errors.Select(e => e.ErrorMessage) ?? [] })
                .ToDictionary(e => e.Key, e => e.Value);
        }

        public BadRequest(string message) : base(
            $"Request body is invalid for {message}.")
        {
        }
    }

    [Error(HttpStatusCode.NotFound, "USER_NOT_FOUND", "User {user_id} not found.")]
    public class UserNotFound(Ulid user_id) :
        ApiError($"User {user_id} not found.");

    [Error(HttpStatusCode.BadRequest, "INVALID_GRANT_TYPE", "Invalid grant type {grant_type}.")]
    public class InvalidGrantType(string grant_type) :
        ApiError($"Invalid grant type {grant_type}.");

    [Error(HttpStatusCode.Unauthorized, "INVALID_TOKEN", "Invalid token {oauth_code}: {oauth_desc}.")]
    public class InvalidToken(string? oauth_code, string? oauth_desc, Exception? ex) :
        ApiError($"Invalid token {oauth_code}: {oauth_desc}.", ex);

    [Error(HttpStatusCode.Forbidden, "INVALID_REFRESH_TOKEN", "Refresh token is not valid for {code}.")]
    public class InvalidRefreshToken(string code) :
        ApiError($"Refresh token is not valid for {code}.");

    [Error(HttpStatusCode.Forbidden, "INVALID_DEVICE_CODE", "Device code is not valid for {code}.")]
    public class InvalidDeviceCode(string code) :
        ApiError($"Device code is not valid for {code}.");

    [Error(HttpStatusCode.Forbidden, "INVALID_AUTHORIZATION_CODE", "Authorization code is not valid.")]
    public class InvalidAuthorizationCode() :
        ApiError($"Refresh token is not valid.");

    [Error(HttpStatusCode.NotFound, "EVENT_NOT_FOUND", "Event {event_id} not found.")]
    public class EventNotFound(Ulid event_id) :
        ApiError($"Event {event_id} not found.");

    [Error(HttpStatusCode.NotFound, "EVENT_AIRSPACE_NOT_FOUND", "Event {event_id}'s airspace {airspace_id} not found.")]
    public class EventAirspaceNotFound(Ulid event_id, Ulid airspace_id) :
        ApiError($"Event {event_id}'s airspace {airspace_id} not found.");

    [Error(HttpStatusCode.NotFound, "EVENT_SLOT_NOT_FOUND", "Event {event_id}'s slot {slot_id} not found.")]
    public class EventSlotNotFound(Ulid event_id, Ulid slot_id) :
        ApiError($"Event {event_id}'s slot {slot_id} not found.");

    [Error(HttpStatusCode.NotFound, "EVENT_SLOT_NOT_BOOKED", "Event {event_id}'s slot {slot_id} has not been booked.")]
    public class EventSlotNotBooked(Ulid event_id, Ulid slot_id) :
        ApiError($"Event {event_id}'s slot {slot_id} has not been booked.");

    [Error(HttpStatusCode.Conflict, "EVENT_SLOT_BOOKED", "Event {event_id}'s slot {slot_id} has been booked.")]
    public class EventSlotBooked(Ulid event_id, Ulid slot_id) :
        ApiError($"Event {event_id}'s slot {slot_id} has been booked.");

    [Error(HttpStatusCode.Conflict, "EVENT_BOOK_OVERLAP_TIME", "Current user have an overlapping booking with the slot.")]
    public class EventBookOverlapTime() :
        ApiError($"Current user have an overlapping booking with the slot.");

    [Error(HttpStatusCode.Forbidden, "EVENT_SLOT_BOOKED_BY_ANOTHER_USER", "Event {event_id}'s slot {slot_id} has been booked by another user.")]
    public class EventSlotBookedByAnotherUser(Ulid event_id, Ulid slot_id) :
        ApiError($"Event {event_id}'s slot {slot_id} has been booked by another user.");

    [Error(HttpStatusCode.Forbidden, "EVENT_NOT_IN_BOOKING_TIME", "Event {event_id} is not in booking time.")]
    public class EventNotInBookingTime(Ulid event_id) :
        ApiError($"Event {event_id} is not in booking time.");

    [Error(HttpStatusCode.Forbidden, "FORBIDDEN", "Permission is not sufficient, lacks {roles}.")]
    public class Forbidden(IEnumerable<string> required_roles) :
        ApiError($"Permission is not sufficient, lacks {string.Join(",", required_roles)}.");

    [Error(HttpStatusCode.BadRequest, "INVALID_AIRPORT_ICAO", "Airport ICAO code {code} is invalid.")]
    public class InvalidAirportIcao(string code) :
        ApiError($"Airport ICAO code {code} is invalid.");

    [Error(HttpStatusCode.NotFound, "CALLSIGN_NOT_FOUND", "Callsign {callsign} is not found.")]
    public class CallsignNotFound(string callsign) :
        ApiError($"Callsign {callsign} is not found.");

    [Error(HttpStatusCode.NotFound, "FLIGHT_NOT_FOUND_FOR_CID", "Flight for CID {cid} is not found.")]
    public class FlightNotFoundForCid(string cid) :
        ApiError($"Flight for CID {cid} is not found.");

    [Error(HttpStatusCode.NotFound, "EVENT_SLOT_NOT_FOUND_FOR_USER", "Event {event_id}'s slot not found for {user_id}.")]
    public class EventSlotNotFoundForUser(Ulid event_id, Ulid user_id) :
        ApiError($"Event {event_id}'s slot not found for {user_id}.");
}
