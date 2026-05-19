use std::time::Duration;

use opentelemetry::global;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::settings::OpenTelemetry;

pub struct OpenTelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl OpenTelemetryGuard {
    fn new() -> Self {
        Self {
            tracer_provider: None,
            meter_provider: None,
            logger_provider: None,
        }
    }
}

impl Drop for OpenTelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.tracer_provider.take()
            && let Err(error) = provider.shutdown()
        {
            eprintln!("failed to shut down OpenTelemetry tracer provider: {error}");
        }
        if let Some(provider) = self.meter_provider.take()
            && let Err(error) = provider.shutdown()
        {
            eprintln!("failed to shut down OpenTelemetry meter provider: {error}");
        }
        if let Some(provider) = self.logger_provider.take()
            && let Err(error) = provider.shutdown()
        {
            eprintln!("failed to shut down OpenTelemetry logger provider: {error}");
        }
    }
}

pub fn init(settings: &OpenTelemetry) -> Result<OpenTelemetryGuard, anyhow::Error> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| settings.log_level.clone().into());
    let resource = Resource::builder()
        .with_service_name(settings.service_name.clone())
        .build();
    let mut guard = OpenTelemetryGuard::new();

    let tracer_provider = if settings.tracing.enabled {
        let exporter = build_span_exporter(settings.tracing.endpoint.as_deref())?;
        let provider = SdkTracerProvider::builder()
            .with_resource(resource.clone())
            .with_batch_exporter(exporter)
            .build();
        global::set_tracer_provider(provider.clone());
        Some(provider)
    } else {
        None
    };

    let meter_provider = if settings.metrics.enabled {
        let exporter = build_metric_exporter(settings.metrics.endpoint.as_deref())?;
        let provider = SdkMeterProvider::builder()
            .with_resource(resource.clone())
            .with_periodic_exporter(exporter)
            .build();
        global::set_meter_provider(provider.clone());
        Some(provider)
    } else {
        None
    };

    let logger_provider = if settings.logs.enabled {
        let exporter = build_log_exporter(settings.logs.endpoint.as_deref())?;
        let provider = SdkLoggerProvider::builder()
            .with_resource(resource)
            .with_batch_exporter(exporter)
            .build();
        Some(provider)
    } else {
        None
    };

    let trace_layer = tracer_provider.as_ref().map(|provider| {
        let tracer = provider.tracer(settings.service_name.clone());
        tracing_opentelemetry::layer().with_tracer(tracer)
    });
    let log_layer = logger_provider
        .as_ref()
        .map(OpenTelemetryTracingBridge::new);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(trace_layer)
        .with(log_layer)
        .init();

    guard.tracer_provider = tracer_provider;
    guard.meter_provider = meter_provider;
    guard.logger_provider = logger_provider;

    Ok(guard)
}

fn build_span_exporter(
    endpoint: Option<&str>,
) -> Result<opentelemetry_otlp::SpanExporter, opentelemetry_otlp::ExporterBuildError> {
    let builder = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5));
    match endpoint {
        Some(endpoint) if !endpoint.is_empty() => builder.with_endpoint(endpoint).build(),
        _ => builder.build(),
    }
}

fn build_metric_exporter(
    endpoint: Option<&str>,
) -> Result<opentelemetry_otlp::MetricExporter, opentelemetry_otlp::ExporterBuildError> {
    let builder = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5));
    match endpoint {
        Some(endpoint) if !endpoint.is_empty() => builder.with_endpoint(endpoint).build(),
        _ => builder.build(),
    }
}

fn build_log_exporter(
    endpoint: Option<&str>,
) -> Result<opentelemetry_otlp::LogExporter, opentelemetry_otlp::ExporterBuildError> {
    let builder = opentelemetry_otlp::LogExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_timeout(Duration::from_secs(5));
    match endpoint {
        Some(endpoint) if !endpoint.is_empty() => builder.with_endpoint(endpoint).build(),
        _ => builder.build(),
    }
}
