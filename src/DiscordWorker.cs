using System.Diagnostics;
using System.Reflection;
using Discord;
using Discord.Interactions;
using Discord.WebSocket;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi;

public class DiscordWorker(
    ILogger<DiscordWorker> Logger,
    ILogger<DiscordSocketClient> ClientLogger,
    DiscordSocketClient Client,
    IOptionsMonitor<DiscordWorker.Option> Options,
    IServiceProvider ServiceProvider
) : IHostedService
{
    protected InteractionService Interaction { get; init; } = new(Client.Rest, new InteractionServiceConfig
    {
        DefaultRunMode = RunMode.Sync,
        AutoServiceScopes = true,
    });
    protected readonly static ActivitySource ActivitySource =
        new(typeof(DiscordWorker).FullName ?? throw new ArgumentNullException());

    public async Task StartAsync(CancellationToken ct)
    {
        using var activity = ActivitySource.StartActivity($"DiscordWorker.StartAsync", ActivityKind.Consumer);

        Client.Ready += async () =>
        {
            using var activity = ActivitySource.StartActivity($"DiscordWorker.Client.Ready", ActivityKind.Internal);
            Logger.LogInformation("Discord bot is connected.");
            try
            {
                await Interaction.RegisterCommandsGloballyAsync();
                Logger.LogInformation("Commands registered.");
            }
            catch (Exception e)
            {
                Logger.LogError(e, "Failed to register");
            }
        };
        Client.Log += (message) =>
        {
            if (message.Message == null) return Task.CompletedTask;
            ClientLogger.Log(message.Severity switch
            {
                LogSeverity.Critical => LogLevel.Critical,
                LogSeverity.Error => LogLevel.Error,
                LogSeverity.Warning => LogLevel.Warning,
                LogSeverity.Info => LogLevel.Information,
                LogSeverity.Verbose => LogLevel.Trace,
                LogSeverity.Debug => LogLevel.Debug,
                _ => LogLevel.Information,
            }, "{Message}", message.Message);
            return Task.CompletedTask;
        };
        using (var scope = ServiceProvider.CreateScope())
        {
            await Interaction.AddModulesAsync(assembly: Assembly.GetEntryAssembly(), services: scope.ServiceProvider);
        }

        Client.InteractionCreated += async (x) =>
        {
            _ = Task.Run(async () =>
            {
                Activity.Current = null;
                using var activity = ActivitySource.StartActivity($"DiscordWorker.Client.InteractionCreated", ActivityKind.Server);
                var ctx = new SocketInteractionContext(Client, x);
                try
                {
                    var result = await Interaction.ExecuteCommandAsync(ctx, ServiceProvider);
                    if (!result.IsSuccess)
                    {
                        string message = result.ErrorReason;
                        if (result is ExecuteResult executeResult && executeResult.Exception != null)
                        {
                            Logger.LogError(executeResult.Exception, "Error occurred executing interaction");
                            message = executeResult.Exception.Message;
                            activity?.AddException(executeResult.Exception);
                        }
                        else
                        {
                            Logger.LogError("Error occurred executing interaction {Error} - {ErrorReason}",
                                result.Error, result.ErrorReason);
                        }
                        activity?.SetStatus(ActivityStatusCode.Error, message);
                        if (!ctx.Interaction.HasResponded && result.Error != InteractionCommandError.UnknownCommand)
                        {
                            await ctx.Interaction.RespondAsync($"Error: Internal error: {message}", ephemeral: true);
                        }
                    }
                }
                catch (Exception e)
                {
                    Logger.LogError(e, "Failed to execute interaction command");
                    activity?.SetStatus(ActivityStatusCode.Error, e.Message);
                    activity?.AddException(e);
                }
            }).ConfigureAwait(false);
        };

        if (string.IsNullOrEmpty(Options.CurrentValue.Token))
        {
            Logger.LogWarning("Discord token is not provided. Discord bot will not be started.");
            return;
        }
        Logger.LogInformation("Login to Discord");
        await Client.LoginAsync(TokenType.Bot, Options.CurrentValue.Token);
        Logger.LogInformation("Start processing messages");
        await Client.StartAsync();
    }

    public async Task StopAsync(CancellationToken ct)
    {
        await Client.StopAsync();
    }

    public class Option
    {
        public const string LOCATION = "Discord";

        public required string Token { get; set; }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<DiscordSocketClient>();
        builder.Services.AddHostedService<DiscordWorker>();
        return builder;
    }
}
