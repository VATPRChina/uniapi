using System.Diagnostics;
using System.Diagnostics.Metrics;
using MailKit;
using MailKit.Net.Smtp;
using Microsoft.Extensions.Options;
using MimeKit.Text;

namespace Net.Vatprc.Uniapi.Adapters.EmailAdapter;

public class SmtpEmailAdapter(IOptions<SmtpEmailAdapter.Option> Options, ILogger logger, IMeterFactory meterFactory)
{
    protected readonly static ActivitySource activitySource = new(typeof(SmtpEmailAdapter).FullName ?? throw new ArgumentNullException());

    protected SmtpClient smtpClient { get; init; } = new SmtpClient(new NullProtocolLogger(), meterFactory);

    public async Task SendEmailAsync(string to, EmailBase email, CancellationToken ct = default)
    {
        using var activity = activitySource.StartActivity("SendEmailAsync", ActivityKind.Client);

        var options = Options.Value;

        if (!smtpClient.IsConnected)
        {
            using var activity2 = activitySource.StartActivity("SendEmailAsync.ConnectAsync", ActivityKind.Client);

            await smtpClient.ConnectAsync(options.Server, options.Port, MailKit.Security.SecureSocketOptions.StartTls, ct);
            await smtpClient.AuthenticateAsync(options.Username, options.Password, ct);
        }

        var message = new MimeKit.MimeMessage();
        message.From.Add(MimeKit.MailboxAddress.Parse(options.From));
        message.To.Add(MimeKit.MailboxAddress.Parse(to));
        message.Subject = email.GetSubject();
        message.Body = new MimeKit.TextPart(TextFormat.Plain) { Text = email.GetPlainText() };
        message.Body = new MimeKit.TextPart(TextFormat.Html) { Text = email.GetPlainText() };

        await smtpClient.SendAsync(message, ct);

        logger.LogInformation("Email sent to {To} with subject {Subject}", to, message.Subject);
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<SmtpEmailAdapter>();
        return builder;
    }

    public class Option
    {
        public const string LOCATION = "Email:Smtp";

        public required string Server { get; set; }

        public required int Port { get; set; }

        public required string Username { get; set; }

        public required string Password { get; set; }

        public required string From { get; set; }
    }
}
