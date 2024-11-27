using System.Diagnostics;
using System.Reflection;
using Discord;
using Discord.Interactions;
using Discord.WebSocket;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class FlightWorker(
    ILogger<FlightWorker> Logger,
    IOptionsMonitor<FlightWorker.Option> Options,
    VatsimService VatsimService
) : BackgroundService
{
    protected readonly PeriodicTimer Timer = new(TimeSpan.FromMinutes(Options.CurrentValue.PeriodInSeconds));

    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        Logger.LogInformation("FlightWorker is starting.");
        while (!stoppingToken.IsCancellationRequested && await Timer.WaitForNextTickAsync(stoppingToken))
        {
            var startTime = DateTimeOffset.Now;
            try
            {
                await RunAsync(stoppingToken);
            }
            catch (Exception e)
            {
                Logger.LogError(e, "An error occurred while running FlightWorker at {Time}.", startTime);
                SentrySdk.CaptureException(e, scope =>
                {
                    scope.TransactionName = $"{nameof(FlightWorker)}@{startTime}";
                });
            }
        }
        Logger.LogInformation("FlightWorker is stopping.");
    }

    protected async Task RunAsync(CancellationToken stoppingToken)
    {
        var data = await VatsimService.GetOnlineData();
        foreach (var flight in data.Pilots)
        {

        }
    }

    public class Option
    {
        public const string LOCATION = "Worker:Flight";

        public required uint PeriodInSeconds { get; set; }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddHostedService<FlightWorker>();
        return builder;
    }
}
