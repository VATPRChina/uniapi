using System.Diagnostics;
using Amazon.Runtime;
using Amazon.S3;
using Arinc424;
using Arinc424.Ground;
using Arinc424.Navigation;
using Arinc424.Procedures;
using Arinc424.Waypoints;
using Microsoft.Extensions.Options;
using Net.Vatprc.Uniapi.Services.FlightPlan;
using Net.Vatprc.Uniapi.Services.FlightPlan.Utility;
using Net.Vatprc.Uniapi.Utils;

namespace Net.Vatprc.Uniapi.Adapters;

public class Arinc424NavdataAdapter(IOptions<Arinc424NavdataAdapter.Option> options, ActivitySource activitySource)
{
    public Data424 Data { get; protected set; } = Data424.Create(Meta424.Create(Supplement.V18), [], out _, out _);

    public async Task InitializeAsync(CancellationToken ct = default)
    {
        using var rootActivity = activitySource.StartActivity($"{nameof(Arinc424NavdataAdapter)}.{nameof(InitializeAsync)}");

        var content = new List<string>();
        using (var activity = activitySource.StartActivity("Load", ActivityKind.Client))
        {
            var option = options.Value;
            var credentials = new BasicAWSCredentials(option.AccessKey, option.SecretKey);
            using var s3Client = new AmazonS3Client(credentials, new AmazonS3Config
            {
                ServiceURL = option.ServiceUrl,
                ForcePathStyle = true,
            });
            var response = await s3Client.GetObjectAsync(option.Bucket, option.ArincPath, ct);
            using var responseStream = response.ResponseStream;
            using var streamReader = new StreamReader(responseStream);
            string? line;
            while ((line = await streamReader.ReadLineAsync()) != null)
            {
                content.Add(line);
            }
        }

        using (var activity = activitySource.StartActivity("Parse", ActivityKind.Internal))
        {
            var meta = Meta424.Create(Supplement.V18);
            Data = Data424.Create(meta, content, out var invalid, out var skipped);
        }
    }

    public class Option
    {
        public const string LOCATION = "Navdata:Arinc424";

        public required string ServiceUrl { get; set; }
        public required string AccessKey { get; set; }
        public required string SecretKey { get; set; }
        public required string Bucket { get; set; }
        public required string ArincPath { get; set; }
    }

    public static WebApplicationBuilder ConfigureOn(WebApplicationBuilder builder)
    {
        builder.Services.Configure<Option>(builder.Configuration.GetSection(Option.LOCATION));
        builder.Services.AddSingleton<Arinc424NavdataAdapter>();
        builder.Services.AddHostedService<Arinc424Initializer>();
        builder.Services.AddScoped<INavdataProvider, Arinc424NavdataProvider>();
        return builder;
    }

    public class Arinc424Initializer : IHostedService
    {
        readonly Arinc424NavdataAdapter _adapter;
        public Arinc424Initializer(Arinc424NavdataAdapter adapter) => _adapter = adapter;

        public Task StartAsync(CancellationToken ct) => _adapter.InitializeAsync(ct);
        public Task StopAsync(CancellationToken ct) => Task.CompletedTask;
    }
}
