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

using System.CommandLine;
using System.Text.Json.Serialization;
using Net.Vatprc.Uniapi.Services;
using Net.Vatprc.Uniapi.Utils;
using Net.Vatprc.Uniapi.Utils.Toml;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc.Formatters;
using Microsoft.IdentityModel.Tokens;
using Microsoft.OpenApi.Models;
using Npgsql;
using Serilog;

Log.Logger = new LoggerConfiguration()
    .Enrich.FromLogContext()
    .Enrich.WithCorrelationId()
    .WriteTo.Console(
        outputTemplate: "[{Timestamp:HH:mm:ss} {Level:u3}] {Message} (at {SourceContext}){NewLine}{Exception}"
    )
    .CreateBootstrapLogger();

var builder = WebApplication.CreateBuilder(args);

builder.WebHost.UseSentry();

builder.Configuration.ReplaceJsonWithToml();
builder.Configuration.AddTomlFile("appsettings.Local.toml", true);

builder.Host.UseSerilog((context, services, configuration) => configuration
    .ReadFrom.Configuration(context.Configuration)
    .ReadFrom.Services(services)
    .Enrich.FromLogContext()
    .Enrich.WithCorrelationId()
);

builder.Services.AddHttpContextAccessor();

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
        options.JsonSerializerOptions.DictionaryKeyPolicy = new JsonSnakeCaseNamingPolicy();
        options.JsonSerializerOptions.PropertyNamingPolicy = new JsonSnakeCaseNamingPolicy();
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

builder.Services.AddSwaggerGen(opts =>
{
    opts.SwaggerDoc("v1", new OpenApiInfo
    {
        Version = "v1",
        Title = "VATPRC UniAPI",
        Description = """
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
        """,
    });
    opts.AddServer(new OpenApiServer { Url = "https://uniapi.vatprc.net", Description = "Public server" });
    opts.AddServer(new OpenApiServer { Url = "https://localhost:5001", Description = "Local development server" });
    opts.AddSecurityDefinition("oauth2_1p", new OpenApiSecurityScheme
    {
        Type = SecuritySchemeType.OAuth2,
        Flows = new OpenApiOAuthFlows
        {
            Password = new OpenApiOAuthFlow
            {
                Scopes = new Dictionary<string, string>
                {
                    { TokenService.JwtScopes.OAuth, "Generate other OAuth tokens." },
                },
                TokenUrl = new Uri("{{baseUrl}}/api/session", UriKind.Relative),
            }
        },
    });
    opts.AddSecurityRequirement(new OpenApiSecurityRequirement
    {
        {
            new OpenApiSecurityScheme
            {
                Reference = new OpenApiReference { Type = ReferenceType.SecurityScheme, Id = "oauth2_1p" }
            },
            Array.Empty<string>()
        }
    });
    opts.IncludeXmlComments(Path.Combine(
        AppContext.BaseDirectory,
        $"{System.Reflection.Assembly.GetExecutingAssembly().GetName().Name}.xml"));
    opts.SupportNonNullableReferenceTypes();
    opts.SchemaFilter<RequiredNotNullableSchemaFilter>();
    opts.OperationFilter<SecurityRequirementsOperationFilter>();
    opts.OperationFilter<ApiError.ErrorResponsesOperationFilter>();
    opts.MapType<Ulid>(() => new OpenApiSchema { Type = "string" });
});

TokenService.ConfigureOn(builder);
builder.Services.AddTransient<AuthenticationEventHandler>();

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

var app = builder.Build();

// Configure the HTTP request pipeline.
if (app.Environment.IsDevelopment())
{
    app.UseDeveloperExceptionPage();
    app.UseSwagger(c => c.RouteTemplate = "/api/swagger/{documentName}/swagger.json");
    app.UseReDoc(c =>
    {
        c.RoutePrefix = "api/swagger";
        c.SpecUrl = "/api/swagger/v1/swagger.json";
    });
}
else
{
    app.UseHsts();
}
app.UseForwardedHeaders();

app.UseSentryTracing();
app.UseSerilogRequestLogging();

if (app.Environment.IsProduction()) app.UseHttpsRedirection();

app.UseRouting();

app.UseCors();

app.UseAuthentication();
app.UseAuthorization();

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
