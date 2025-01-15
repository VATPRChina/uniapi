using System.Diagnostics;
using System.Reflection;
using Discord;
using Discord.Interactions;
using Discord.WebSocket;
using Microsoft.Extensions.Options;

namespace Net.Vatprc.Uniapi.Services;

public class DiscordWorker(
    ILogger<DiscordWorker> Logger,
    ILogger<DiscordSocketClient> ClientLogger,
    DiscordSocketClient Client,
    IOptionsMonitor<DiscordWorker.Option> Options,
    IServiceProvider ServiceProvider
) : IHostedService
{
    protected InteractionService Interaction { get; init; } = new(Client.Rest);
    protected readonly static ActivitySource ActivitySource =
        new(typeof(DiscordWorker).FullName ?? throw new ArgumentNullException());

    public async Task StartAsync(CancellationToken ct)
    {
        Client.Ready += async () =>
        {
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
        await Interaction.AddModulesAsync(assembly: Assembly.GetEntryAssembly(), services: ServiceProvider);

        Client.InteractionCreated += async (x) =>
        {
            using var activity = ActivitySource.StartActivity($"Discord {x.Type}", ActivityKind.Server);
            var ctx = new SocketInteractionContext(Client, x);
            await Interaction.ExecuteCommandAsync(ctx, ServiceProvider);
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
