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
using Net.Vatprc.Uniapi.Controllers;
using Net.Vatprc.Uniapi.Controllers.Atc;
using Net.Vatprc.Uniapi.Models.Sheet;
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
        if (knownNetwork.Value is null) continue;
        options.KnownIPNetworks.Add(System.Net.IPNetwork.Parse(knownNetwork.Value));
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
        .AddSource("Net.Vatprc.Uniapi.*")
        .AddAspNetCoreInstrumentation()
        .AddEntityFrameworkCoreInstrumentation()
        .AddHttpClientInstrumentation()
        .AddOtlpExporter("Tracing", configure: null))
    .WithMetrics(metrics => metrics
        .AddMeter("Net.Vatprc.Uniapi.*")
        .AddAspNetCoreInstrumentation()
        .AddHttpClientInstrumentation()
        .AddOtlpExporter("Metrics", configure: null));

builder.Services.AddHttpContextAccessor();

builder.Services.ConfigureHttpJsonOptions(options =>
{
    options.SerializerOptions.DictionaryKeyPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower;
    options.SerializerOptions.Converters.Add(new JsonStringEnumConverter(JsonNamingPolicy.KebabCaseLower));
    options.SerializerOptions.NumberHandling = JsonNumberHandling.Strict;
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
    opts.AddDocumentTransformer(OpenApiTransformers.AddUlid);
    opts.AddSchemaTransformer(OpenApiTransformers.AnnotateUlid);
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
builder.Services.AddScoped<AtcApplicationService>();

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
    var sheetService = scope.ServiceProvider.GetRequiredService<SheetService>();

    await db.Database.MigrateAsync();

    await sheetService.SetSheetFieldsAsync(UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
        [
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "age",
                Kind = SheetFieldKind.ShortText,
                Sequence = 1,
                NameZh = "年龄",
                NameEn = "Age",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "occupation",
                Kind = SheetFieldKind.ShortText,
                Sequence = 2,
                NameZh = "职业",
                NameEn = "Occupation",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "location",
                Kind = SheetFieldKind.ShortText,
                Sequence = 3,
                NameZh = "现居城市",
                NameEn = "City of Residence",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "previous_experience",
                Kind = SheetFieldKind.SingleChoice,
                SingleChoiceOptions = [
                    "有（如果可能，请在下方的简介中描述）",
                    "无",
                ],
                Sequence = 4,
                NameZh = "虚拟管制经验",
                NameEn = "Virtual ATC Experience",
                DescriptionZh = "是否有过虚拟平台的管制经验？",
                DescriptionEn = "Do you have experience as a virtual air traffic controller?",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "weekly_hours",
                Kind = SheetFieldKind.ShortText,
                Sequence = 5,
                NameZh = "每周可服务小时数",
                NameEn = "Weekly Available Hours",
                DescriptionZh = "自评每周可以提供管制服务的小时数",
                DescriptionEn = "Estimate the number of hours you can provide ATC services per week",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "english_level",
                Kind = SheetFieldKind.SingleChoice,
                SingleChoiceOptions = [
                    "不懂英文或几乎不懂英文",
                    "粗通英文，可以借助文字和翻译器进行英文陆空对话",
                    "勉强能听懂和进行陆空对话",
                    "能听懂大多数场景下的陆空对话，能用语音进行流利的英语对话",
                    "除了熟练运用陆空对话外，还能用英文处理一些非常规情景的对话。",
                    "英文交流基本无任何障碍",
                ],
                Sequence = 6,
                NameZh = "英语水平",
                NameEn = "English Proficiency",
                DescriptionZh = "你的英语水平如何？",
                DescriptionEn = "How would you rate your English proficiency?",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "self_introduction",
                Kind = SheetFieldKind.LongText,
                Sequence = 7,
                NameZh = "个人模拟飞行经历简介",
                NameEn = "Flight Simulation Experience",
                DescriptionZh = "可用一段话简单介绍自己和模拟飞行/VATSIM相关的经历，对不同模拟飞行平台的了解等，建议约50字左右。请详细列写参加过的活动名称与对应的机组呼号，相关职员将对其进行检查。",
                DescriptionEn = "You can use a paragraph to briefly introduce yourself and your experience related to flight simulation/VATSIM, as well as your understanding of different flight simulation platforms. It is recommended to keep it around 50 words. Please list the names of the events you have participated in along with the corresponding callsigns, as the relevant staff will check them.",
            },
            new SheetField
            {
                SheetId = UserAtcApplicationController.ATC_APPLICATION_SHEET_ID,
                Id = "expectation",
                Kind = SheetFieldKind.LongText,
                Sequence = 8,
                NameZh = "期望收获",
                NameEn = "Expectations",
                DescriptionZh = "可用一段话简单介绍自己申请成为管制员的原因和期望收获，建议约50字左右。",
                DescriptionEn = "You can use a paragraph to briefly introduce your reasons for applying to become an air traffic controller and your expectations, recommended to be around 50 words.",
            },
        ]);
    await sheetService.SetSheetFieldsAsync(AtcApplicationController.ATC_APPLICATION_REVIEW_SHEET_ID,
        [
            new SheetField
            {
                SheetId = AtcApplicationController.ATC_APPLICATION_REVIEW_SHEET_ID,
                Id = "review",
                Kind = SheetFieldKind.LongText,
                Sequence = 1,
                NameZh = "面试评价",
                NameEn = "Review Comments",
            },
        ]);
});
rootCommand.Add(new NavdataCommand(app));

await rootCommand.InvokeAsync(args);
