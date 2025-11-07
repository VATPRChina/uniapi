using System.Diagnostics;
using OpenTelemetry;
using OpenTelemetry.Context.Propagation;

namespace Net.Vatprc.Uniapi.Utils;

public class TraceparentMiddleware
{
    private readonly RequestDelegate _next;
    private readonly TextMapPropagator _propagator;

    public TraceparentMiddleware(RequestDelegate next)
    {
        _next = next;
        _propagator = Propagators.DefaultTextMapPropagator;
    }

    public async Task InvokeAsync(HttpContext context)
    {
        var activity = Activity.Current;
        if (activity != null)
        {
            _propagator.Inject(new PropagationContext(activity.Context, Baggage.Current),
                context.Response.Headers,
                (headers, key, value) => headers[key] = value);
        }

        await _next(context);
    }
}
