global using Microsoft.EntityFrameworkCore;
global using Net.Vatprc.Uniapi;
global using static Net.Vatprc.Uniapi.Utils.Utils;
global using UniApi = Net.Vatprc.Uniapi;
using System.CommandLine;
using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.HttpOverrides;
using Microsoft.AspNetCore.Mvc.Formatters;
using Microsoft.IdentityModel.Tokens;
using Net.Vatprc.Uniapi.Adapters;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Services.FlightPlan.Parsing;
using Net.Vatprc.Uniapi.Utils;
using Net.Vatprc.Uniapi.Utils.Toml;
using Npgsql;
using OpenTelemetry.Exporter;
using OpenTelemetry.Logs;
using OpenTelemetry.Metrics;
using OpenTelemetry.Resources;
using OpenTelemetry.Trace;
using Scalar.AspNetCore;

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.ReplaceJsonWithToml();
builder.Configuration.AddTomlFile("appsettings.Local.toml", true);

builder.Services.Configure<ForwardedHeadersOptions>(options =>
{
    options.ForwardedHeaders =
        ForwardedHeaders.XForwardedFor | ForwardedHeaders.XForwardedProto;
    options.ForwardLimit =
        builder.Configuration.GetSection("ForwardedHeadersOptions").GetValue("ForwardLimit", 1);
    foreach (var knownNetwork in builder.Configuration
        .GetSection("ForwardedHeadersOptions:KnownNetworks").GetChildren())
    {
        options.KnownNetworks.Add(IPNetwork.Parse(knownNetwork.Value));
    }
});

builder.Services.Configure<OtlpExporterOptions>("Tracing", builder.Configuration.GetSection("OpenTelemetry:Tracing:Otlp"));
builder.Services.Configure<OtlpExporterOptions>("Metrics", builder.Configuration.GetSection("OpenTelemetry:Metrics:Otlp"));
builder.Services.Configure<OtlpExporterOptions>("Logging", builder.Configuration.GetSection("OpenTelemetry:Logging:Otlp"));
var resource = ResourceBuilder.CreateDefault().AddService(
    serviceName: "vatprc-uniapi",
    serviceNamespace: "vatprc",
    serviceVersion: Assembly.GetExecutingAssembly().GetName().Version?.ToString());
builder.Logging.AddOpenTelemetry(options =>
{
    options.IncludeFormattedMessage = builder.Configuration
        .GetValue("OpenTelemetry:Logging:IncludeFormattedMessage", true);
    options
        .SetResourceBuilder(resource)
        .AddOtlpExporter("Logging", configure: null);
});
builder.Services.AddOpenTelemetry()
    .ConfigureResource(resource => resource.AddService(serviceName: "vatprc-uniapi",
        serviceNamespace: "vatprc",
        serviceVersion: Assembly.GetExecutingAssembly().GetName().Version?.ToString()))
    .WithTracing(tracing => tracing
        .AddAspNetCoreInstrumentation()
        .AddEntityFrameworkCoreInstrumentation()
        .AddHttpClientInstrumentation()
        .AddOtlpExporter("Tracing", configure: null))
    .WithMetrics(metrics => metrics
        .AddAspNetCoreInstrumentation()
        .AddHttpClientInstrumentation()
        .AddOtlpExporter("Metrics", configure: null));

builder.Services.AddHttpContextAccessor();

builder.Services.ConfigureHttpJsonOptions(options =>
{
    options.SerializerOptions.DictionaryKeyPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.Converters.Add(new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower));
});
builder.Services
    .AddControllers(opts =>
    {
        opts.OutputFormatters.RemoveType<StringOutputFormatter>();
        opts.OutputFormatters.RemoveType<StreamOutputFormatter>();
        opts.Filters.Add<ApiError.ErrorExceptionFilter>();

        var jsonInputFormatter = opts.InputFormatters.OfType<SystemTextJsonInputFormatter>().First();
        jsonInputFormatter.SupportedMediaTypes.Remove("text/json");
        jsonInputFormatter.SupportedMediaTypes.Remove("application/*+json");

        var jsonOutputFormatter = opts.OutputFormatters.OfType<SystemTextJsonOutputFormatter>().First();
        jsonOutputFormatter.SupportedMediaTypes.Remove("text/json");
        jsonOutputFormatter.SupportedMediaTypes.Remove("application/*+json");
    })
    .AddJsonOptions(options =>
    {
        options.JsonSerializerOptions.DictionaryKeyPolicy = JsonNamingPolicy.SnakeCaseLower;
        options.JsonSerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower;
        options.JsonSerializerOptions.Converters.Add(new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower));
    })
    .ConfigureApiBehaviorOptions(options =>
    {
        options.InvalidModelStateResponseFactory = context =>
            throw new ApiError.BadRequest(context.ModelState);
    });
builder.Services.AddProblemDetails();
builder.Services.AddHttpContextAccessor();

var connectionString = builder.Configuration.GetConnectionString("VATPRCContext") ??
    throw new Exception("Connection string for VATPRCContext cannot be null");
var dataSource = new NpgsqlDataSourceBuilder(connectionString)
    .EnableDynamicJson()
    .Build();
builder.Services.AddDbContext<Database>(opt =>
{
    opt.UseNpgsql(dataSource);
});

builder.Services.AddOpenApi(opts =>
{
    opts.AddDocumentTransformer(OpenApiTransformers.TransformDocument);
    opts.AddSchemaTransformer(OpenApiTransformers.AddUlid);
    opts.AddSchemaTransformer(OpenApiTransformers.EnforceNotNull);
    opts.AddOperationTransformer(OpenApiTransformers.AllowAnonymous);
    opts.AddOperationTransformer(OpenApiTransformers.AddErrorResponse);
});

TokenService.ConfigureOn(builder);
builder.Services.AddTransient<AuthenticationEventHandler>();

VatsimAuthAdapter.ConfigureOn(builder);

builder.Services.AddAuthentication(opts =>
{
    opts.DefaultChallengeScheme = JwtBearerDefaults.AuthenticationScheme;
    opts.DefaultSignInScheme = JwtBearerDefaults.AuthenticationScheme;
    opts.DefaultAuthenticateScheme = JwtBearerDefaults.AuthenticationScheme;
})
.AddJwtBearer(opts =>
{
    opts.SaveToken = true;
    var config = builder.Configuration.GetSection(TokenService.Option.LOCATION)
        .Get<TokenService.Option>();
    new TokenService.OptionConfigure().Configure(config!);
    opts.TokenValidationParameters = new TokenValidationParameters
    {
        ValidIssuer = config?.Issuer,
        ValidateIssuer = true,
        ValidateAudience = false,
        IssuerSigningKey = config?.SecurityKey,
        ValidateIssuerSigningKey = true,
        ValidateLifetime = true,
        ClockSkew = TimeSpan.FromSeconds(30),
    };
    opts.EventsType = typeof(AuthenticationEventHandler);
});
builder.Services.AddAuthorization();
builder.Services.AddSingleton<IAuthorizationMiddlewareResultHandler, AuthorizationHandler>();

DiscordWorker.ConfigureOn(builder);

builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.WithOrigins("https://*.vatprc.net")
            .SetIsOriginAllowedToAllowWildcardSubdomains();
        if (builder.Environment.IsDevelopment())
        {
            policy.AllowAnyOrigin();
        }
        policy.AllowAnyMethod()
            .WithHeaders("authorization", "content-type");
    });
});

MetarAdapter.ConfigureOn(builder);
builder.Services.AddSingleton<VatsimAdapter>();
VatprcAtcApiAdapter.ConfigureOn(builder);
DiscourseAdapter.ConfigureOn(builder);
FlightWorker.ConfigureOn(builder);
builder.Services.AddSingleton<TrackAudioAdapter>();
builder.Services.AddScoped<DbNavdataAdapter>();
builder.Services.AddScoped<RouteParseService>();
builder.Services.AddMemoryCache();
builder.Services.AddSingleton<RouteParserFactory>();
SmmsAdapter.ConfigureOn(builder);
builder.Services.AddScoped<SheetService>();
builder.Services.AddScoped<IUserAccessor, UserAccessor>();
builder.Services.AddSingleton<AtcPositionKindService>();
builder.Services.AddSingleton<AtcPositionStatusService>();

var app = builder.Build();

app.UseExceptionHandler();

// Configure the HTTP request pipeline.
app.MapOpenApi();
if (app.Environment.IsDevelopment())
{
    app.MapScalarApiReference();
}
else
{
    app.UseHsts();
}
app.UseForwardedHeaders();

if (app.Environment.IsProduction()) app.UseHttpsRedirection();

app.UseMiddleware<TraceparentMiddleware>();

app.UseRouting();

app.UseCors();

app.UseAuthentication();
app.UseAuthorization();

app.UseFileServer();

app.MapControllers().RequireAuthorization(new AuthorizationPolicyBuilder().RequireAuthenticatedUser().Build());

app.MapFallbackToController("/api/{**path}",
    nameof(UniApi.Controllers.InternalController.EndpointNotFound),
    nameof(UniApi.Controllers.InternalController).Replace("Controller", ""));

var rootCommand = new RootCommand("Start VATPRC UniAPI");
rootCommand.SetHandler(async () =>
{
    if (app.Environment.IsDevelopment())
    {
        app.Services.GetRequiredService<ILogger<Program>>().LogInformation("See API document on: https://localhost:5001/scalar/v1");
    }
    await app.RunAsync();
});
rootCommand.TreatUnmatchedTokensAsErrors = false;

var migrateCommand = new Command("migrate", "Migrate database");
rootCommand.Add(migrateCommand);
migrateCommand.SetHandler(async () =>
{
    using var scope = app.Services.CreateScope();
    using var db = scope.ServiceProvider.GetRequiredService<Database>();
    await db.Database.MigrateAsync();
});
rootCommand.Add(new NavdataCommand(app));

await rootCommand.InvokeAsync(args);
