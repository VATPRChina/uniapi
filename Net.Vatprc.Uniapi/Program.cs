global using Microsoft.EntityFrameworkCore;
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
using System.Text.Json;
using nietras.SeparatedValues;

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
builder.Services.AddProblemDetails();

var connectionString = builder.Configuration.GetConnectionString(nameof(VATPRCContext)) ??
    throw new Exception("Connection string for VATPRCContext cannot be null");
var dataSource = new NpgsqlDataSourceBuilder(connectionString)
    .EnableDynamicJson()
    .Build();
builder.Services.AddDbContext<VATPRCContext>(opt =>
{
    opt.UseSnakeCaseNamingConvention();
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
            policy.AllowAnyOrigin();
        }
        policy.AllowAnyMethod()
            .WithHeaders("authorization", "content-type");
    });
});

RudiMetarService.ConfigureOn(builder);
VatsimService.ConfigureOn(builder);
VatprcAtcService.ConfigureOn(builder);
DiscourseService.ConfigureOn(builder);
FlightWorker.ConfigureOn(builder);

var app = builder.Build();

app.UseExceptionHandler();

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
    using var db = scope.ServiceProvider.GetRequiredService<VATPRCContext>();
    await db.Database.MigrateAsync();
});

var navdataCommand = new Command("import-navdata", "Import navdata");
rootCommand.Add(navdataCommand);
navdataCommand.SetHandler(async () =>
{
    using var scope = app.Services.CreateScope();
    using var db = scope.ServiceProvider.GetRequiredService<VATPRCContext>();

    await db.Airport.ExecuteDeleteAsync();
    await db.Runway.ExecuteDeleteAsync();
    await db.AirportPhysicalRunway.ExecuteDeleteAsync();
    await db.AirportGate.ExecuteDeleteAsync();

    var airports = new List<UniApi.Models.Navdata.Airport>();
    var runways = new List<UniApi.Models.Navdata.Runway>();
    var physicalRunways = new List<UniApi.Models.Navdata.AirportPhysicalRunway>();
    var gates = new List<UniApi.Models.Navdata.AirportGate>();
    foreach (var line in Sep.Reader().FromFile("2412-data/AD_HP.csv"))
    {
        if (line["VAL_LONG_RWY"].ToString() == "0") continue;
        var airport = new UniApi.Models.Navdata.Airport
        {
            Identifier = line["CODE_ID"].ToString(),
            Latitude = Geography.ParseCaacCoordinate(line["GEO_LAT_ACCURACY"].ToString()),
            Longitude = Geography.ParseCaacCoordinate(line["GEO_LONG_ACCURACY"].ToString()),
            Elevation = Geography.ConvertMeterToFeet(line["VAL_ELEV"].Parse<int>()),
        };
        airports.Add(airport);
    }
    foreach (var line in File.ReadLines("Runway.txt"))
    {
        // 01 19 40.05888889 116.6176111 40.09286111 116.6122778 ZBAA
        var data = line.Split(' ');
        var airport = airports.FirstOrDefault(a => a.Identifier == data[^1].Trim('"'));
        if (airport == null)
        {
            Console.WriteLine($"Airport {data[0]} not found");
            continue;
        }
        var runway1 = new UniApi.Models.Navdata.Runway
        {
            Identifier = data[0],
            Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
            Latitude = double.Parse(data[2]),
            Longitude = double.Parse(data[3]),
        };
        var runway2 = new UniApi.Models.Navdata.Runway
        {
            Identifier = data[1],
            Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
            Latitude = double.Parse(data[4]),
            Longitude = double.Parse(data[5]),
        };
        var physicalRunway = new UniApi.Models.Navdata.AirportPhysicalRunway
        {
            Airport = airports.FirstOrDefault(a => a.Identifier == data[^1]),
            Runway1 = runway1,
            Runway2 = runway2,
        };
        runways.Add(runway1);
        runways.Add(runway2);
        physicalRunways.Add(physicalRunway);
    }
    foreach (var line in File.ReadLines("sectors_stand.csv"))
    {
        // "ZBAA","A106","40.082","116.5835","61","",""
        var data = line.Split(',');
        var airport = airports.FirstOrDefault(a => a.Identifier == data[0].Trim('"'));
        if (airport == null)
        {
            Console.WriteLine($"Airport {data[0]} not found");
            continue;
        }
        var gate = new UniApi.Models.Navdata.AirportGate
        {
            Airport = airports.FirstOrDefault(a => a.Identifier == data[0].Trim('"')),
            Identifier = data[1].Trim('"'),
            Latitude = double.Parse(data[2].Trim('"')),
            Longitude = double.Parse(data[3].Trim('"')),
        };
        gates.Add(gate);
    }

    db.Airport.AddRange(airports);
    db.Runway.AddRange(runways);
    db.AirportPhysicalRunway.AddRange(physicalRunways);
    db.AirportGate.AddRange(gates);
    await db.SaveChangesAsync();
});

await rootCommand.InvokeAsync(args);
