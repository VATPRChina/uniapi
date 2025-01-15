
using System.Reactive.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.RegularExpressions;
using Flurl.Http;
using Microsoft.Extensions.Options;
using Websocket.Client;

namespace Net.Vatprc.Uniapi.Services;

public class QQWorker(
    IOptionsMonitor<QQWorker.Option> Options,
    VatsimService VatsimService,
    ILogger<QQWorker> Logger
) : IHostedService
{
    protected WebsocketClient? Client { get; set; }

    public async Task StartAsync(CancellationToken cancellationToken)
    {
        if (Options.CurrentValue.Uri == null)
        {
            Logger.LogWarning("QQ WebSocket URI is not configured, worker will not start.");
            return;
        }

        Options.OnChange(async (option, _) =>
        {
            if (Client == null) return;
            Client.Url = new Uri(option.Uri); ;
            await Client.Reconnect();
        });

        Client = new(new(Options.CurrentValue.Uri));
        Client.MessageReceived
            .Where(msg => msg.Text != null)
            .Subscribe(OnMessage);
        await Client.Start();
    }

    public async Task StopAsync(CancellationToken cancellationToken)
    {
        if (Client != null)
        {
            await Client.Stop(System.Net.WebSockets.WebSocketCloseStatus.NormalClosure, "Stop");
        }
    }

    public class GroupJoinMessage
    {
        [JsonPropertyName("time")]
        public long Time { get; set; }

        [JsonPropertyName("self_id")]
        public long SelfId { get; set; }

        [JsonPropertyName("post_type")]
        public string PostType { get; set; } = string.Empty;

        [JsonPropertyName("group_id")]
        public long GroupId { get; set; }

        [JsonPropertyName("user_id")]
        public long UserId { get; set; }

        [JsonPropertyName("request_type")]
        public string RequestType { get; set; } = string.Empty;

        [JsonPropertyName("comment")]
        public string Comment { get; set; } = string.Empty;

        [JsonPropertyName("flag")]
        public string Flag { get; set; } = string.Empty;

        [JsonPropertyName("sub_type")]
        public string SubType { get; set; } = string.Empty;
    }

    protected void SetGroupAddRequestApprove(string subtype, string flag)
    {
        Client?.Send($$"""
            {
                "action": "set_group_add_request",
                "params": {
                    "flag": "{{flag}}",
                    "sub_type": "{{subtype}}",
                    "approve": true,
                    "reason": ""
                }
            }
            """);
    }

    protected void SetGroupAddRequestDeny(string flag, string subtype, string reason)
    {
        Client?.Send($$"""
            {
                "action": "set_group_add_request",
                "params": {
                    "flag": "{{flag}}",
                    "sub_type": "{{subtype}}",
                    "approve": false,
                    "reason": "{{reason}}"
                }
            }
            """);
    }

    protected void OnMessage(ResponseMessage message)
    {
        Task.Run(() => OnMessageAsync(message)).GetAwaiter().GetResult();
    }

    protected async Task OnMessageAsync(ResponseMessage message)
    {
        if (message.Text == null) return;

        var payload = JsonSerializer.Deserialize<GroupJoinMessage>(message.Text);
        if (payload == null || payload.PostType != "request" || payload.RequestType != "group")
        {
            return;
        }

        if (payload.Comment.Contains("人工")) return;
        if (payload.GroupId.ToString() != Options.CurrentValue.GroupId) return;

        Logger.LogInformation("Received group join request: {Comment} for QQ {QQ}", payload.Comment, payload.UserId);
        var cid = new Regex(@"\d+").Match(payload.Comment).Value;
        if (string.IsNullOrWhiteSpace(cid))
        {
            Logger.LogInformation("Deny group join request: {Comment} since no CID", payload.Comment);
            SetGroupAddRequestDeny(payload.Flag, payload.SubType, "加群申请中不包含CID");
            return;
        }

        try
        {
            var userInfo = await VatsimService.GetUserInfo(cid);
            if (userInfo.Rating <= 0)
            {
                Logger.LogInformation("Deny group join request: {Cid} since user rating is 0 or -1", cid);
                SetGroupAddRequestDeny(payload.Flag, payload.SubType, "VATSIM用户状态异常");
                return;
            }
            SetGroupAddRequestApprove(payload.SubType, payload.Flag);
        }
        catch (FlurlHttpException e) when (e.StatusCode == (int)System.Net.HttpStatusCode.NotFound)
        {
            Logger.LogInformation("Deny group join request: {Cid} since user not found", cid);
            SetGroupAddRequestDeny(payload.Flag, payload.SubType, "未查询到CID相关用户");
            return;
        }
        catch (Exception e)
        {
            Logger.LogError(e, "Failed to process group join request");
            e.SetSentryMechanism(nameof(QQWorker), nameof(OnMessage), false);
            SentrySdk.CaptureException(e);
        }
    }

    public class Option
    {
        public const string LOCATION = "QQ";

        public required string Uri { get; set; }

        public required string GroupId { get; set; }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddHostedService<QQWorker>();
        return builder;
    }
}
