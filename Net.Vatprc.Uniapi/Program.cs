global using Microsoft.EntityFrameworkCore;
global using i32 = int;
global using u32 = uint;
global using i64 = long;
global using u64 = ulong;
global using i8 = char;
global using u8 = byte;
global using i16 = short;
global using u16 = ushort;
global using f32 = float;
global using f64 = double;
global using Net.Vatprc.Uniapi;
global using UniApi = Net.Vatprc.Uniapi;
global using static Net.Vatprc.Uniapi.Utils.Utils;

using System.CommandLine;
using System.Text.Json.Serialization;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using Net.Vatprc.Uniapi.Utils.Toml;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc.Formatters;
using Microsoft.IdentityModel.Tokens;
using Npgsql;
using Serilog;
using Sentry.OpenTelemetry;
using OpenTelemetry.Trace;
using Microsoft.AspNetCore.HttpOverrides;
using Scalar.AspNetCore;
using Microsoft.OpenApi.Models;
using System.Text.Json;
using Microsoft.AspNetCore.Http.Json;

Log.Logger = new LoggerConfiguration()
    .Enrich.FromLogContext()
    .Enrich.WithClientIp()
    .Enrich.WithCorrelationId(addValueIfHeaderAbsence: true)
    .WriteTo.Console(
        outputTemplate: "[{Timestamp:HH:mm:ss} {Level:u3} {CorrelationId}] {Message} (at {SourceContext}){NewLine}{Exception}"
    )
    .CreateBootstrapLogger();

var builder = WebApplication.CreateBuilder(args);

builder.WebHost.UseSentry(opts => opts.UseOpenTelemetry());

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

builder.Host.UseSerilog((context, services, configuration) => configuration
    .Enrich.FromLogContext()
    .Enrich.WithClientIp()
    .Enrich.WithCorrelationId(addValueIfHeaderAbsence: true)
    .ReadFrom.Configuration(context.Configuration)
    .ReadFrom.Services(services)
);

builder.Services.AddOpenTelemetry()
    .WithTracing(tracerProviderBuilder =>
        tracerProviderBuilder
            .AddSource($"{System.Reflection.Assembly.GetExecutingAssembly().GetName().Name}.*")
            .AddAspNetCoreInstrumentation()
            .AddEntityFrameworkCoreInstrumentation()
            .AddHttpClientInstrumentation()
            .AddSentry()
    );

builder.Services.AddHttpContextAccessor();

builder.Services.ConfigureHttpJsonOptions(options =>
{
    options.SerializerOptions.DictionaryKeyPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.Converters.Add(new JsonStringEnumConverter());
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
        options.JsonSerializerOptions.Converters.Add(new JsonStringEnumConverter());
    })
    .ConfigureApiBehaviorOptions(options =>
    {
        options.InvalidModelStateResponseFactory = context =>
            throw new ApiError.BadRequest(context.ModelState);
    });

var connectionString = builder.Configuration.GetConnectionString(nameof(VATPRCContext)) ??
    throw new Exception("Connection string for VATPRCContext cannot be null");
var dataSource = new NpgsqlDataSourceBuilder(connectionString)
    .EnableDynamicJson()
    .Build();
builder.Services.AddDbContext<VATPRCContext>(opt =>
{
    opt.UseNpgsql(dataSource);
    opt.UseSnakeCaseNamingConvention();
});

builder.Services.AddOpenApi(opts =>
{
    opts.AddDocumentTransformer((document, context, cancellationToken) =>
    {
        document.Info.Title = "VATPRC UniAPI";
        document.Info.Version = "v1";
        document.Info.Description = """
        # Error Handling

        VATPRC UniAPI returns normalized error responses. The response body is a JSON object with the following fields:

        | Field           | Type     | Description     |
        | --------------- | -------- | --------------- |
        | `error_code`    | `string` | Error code.     |
        | `message`       | `string` | Error message.  |
        | `connection_id` | `string` | Connection ID.     |
        | `request_id`    | `string` | Request ID. |

        It may contain additional fields depending on the error code.

        For details, see the examples on each API endpoint. The additional fields is denoted like `{field}` in the
        error message example.
        """;
        document.Servers.Add(new OpenApiServer
        {
            Url = "https://uniapi.vatprc.net",
            Description = "Production server"
        });
        document.Components ??= new OpenApiComponents();
        document.Components.SecuritySchemes.Add("oauth2", new OpenApiSecurityScheme
        {
            Type = SecuritySchemeType.OAuth2,
            Flows = new OpenApiOAuthFlows
            {
                Password = new OpenApiOAuthFlow
                {
                    TokenUrl = new Uri("{{baseUrl}}/api/session", UriKind.Relative),
                },
                AuthorizationCode = new OpenApiOAuthFlow
                {
                    AuthorizationUrl = new Uri("{{baseUrl}}/auth/authorize", UriKind.Relative),
                    TokenUrl = new Uri("{{baseUrl}}/auth/token", UriKind.Relative),
                    RefreshUrl = new Uri("{{baseUrl}}/auth/token", UriKind.Relative),
                },
            },
        });
        document.SecurityRequirements.Add(new OpenApiSecurityRequirement
        {
            {
                new OpenApiSecurityScheme
                {
                    Reference = new OpenApiReference { Id = "oauth2", Type = ReferenceType.SecurityScheme },
                },
                []
            }
        });
        return Task.CompletedTask;
    });
    opts.AddSchemaTransformer((schema, context, cancellationToken) =>
    {
        if (context.JsonTypeInfo.Type == typeof(Ulid))
        {
            schema.Type = "string";
            schema.Description = "ULID";
        }
        return Task.CompletedTask;
    });
});

TokenService.ConfigureOn(builder);
builder.Services.AddTransient<AuthenticationEventHandler>();

VatsimAuthService.ConfigureOn(builder);
// FIXME: This will raise "No XML encryptor configured. Key {GUID} may be
// persisted to storage in unencrypted form." on start, but I think it is ok
// as the JWT keys are managed manually.
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
            policy.WithOrigins("http://localhost:3000");
        }
        policy.AllowAnyMethod()
            .WithHeaders("authorization", "content-type");
    });
});

RudiMetarService.ConfigureOn(builder);
VatsimService.ConfigureOn(builder);
VatprcAtcService.ConfigureOn(builder);
DiscourseService.ConfigureOn(builder);

var app = builder.Build();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
    app.MapScalarApiReference();
}
else
{
    app.UseHsts();
}
app.UseForwardedHeaders();

if (app.Environment.IsProduction()) app.UseHttpsRedirection();

app.UseSerilogRequestLogging();

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
    await app.RunAsync();
});
rootCommand.TreatUnmatchedTokensAsErrors = false;

var migrateCommand = new Command("migrate", "Migrate database");
rootCommand.Add(migrateCommand);
migrateCommand.SetHandler(async () =>
{
    using var scope = app.Services.CreateScope();
    using var db = scope.ServiceProvider.GetRequiredService<VATPRCContext>();
    await db.Database.MigrateAsync();
});

await rootCommand.InvokeAsync(args);
