use std::sync::OnceLock;

use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{LogExporter, MetricExporter, SpanExporter};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{Resource, logs::SdkLoggerProvider};
use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use thiserror::Error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::{Layer, SubscriberExt};
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Error, Debug)]
pub enum ObservInitErr {
    #[error("Doesnt initiate")]
    DoesntInit,
}

pub fn init_observ() -> anyhow::Result<()> {
    if !std::env::var("INIT_OBSERV")?.parse::<bool>()? {
        return Err(ObservInitErr::DoesntInit.into());
    }

    // 1) Providers
    let logger_provider = init_logs();
    let tracer_provider = init_traces();
    let meter_provider = init_metrics();

    // 2) Layers
    let logs_filter = EnvFilter::new("info")
        .add_directive("hyper=off".parse()?)
        .add_directive("tonic=off".parse()?)
        .add_directive("h2=off".parse()?)
        .add_directive("reqwest=off".parse()?)
        .add_directive("Pyroscope=off".parse()?);

    let otel_logs_layer =
        OpenTelemetryTracingBridge::new(&logger_provider).with_filter(logs_filter);

    let otel_tracer = tracer_provider.tracer("app");
    let traces_filter = EnvFilter::new("info");
    let otel_traces_layer = tracing_opentelemetry::layer()
        .with_tracer(otel_tracer)
        .with_filter(traces_filter);

    let filter_fmt = EnvFilter::new("info").add_directive("opentelemetry=error".parse()?);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .with_filter(filter_fmt);

    // 3) Подписчик
    tracing_subscriber::registry()
        .with(otel_traces_layer) // спаны -> OTLP -> Jaeger
        .with(otel_logs_layer) // события -> OTLP Logs
        .with(fmt_layer) // печать в stdout
        .init();

    // 4) Сделать провайдеры глобальными (если в другом коде вызывается global::tracer()/meter())
    opentelemetry::global::set_tracer_provider(tracer_provider);
    opentelemetry::global::set_meter_provider(meter_provider);

    // 5) Запускаем pyroscope profiler
    init_pyroscope()?;

    Ok(())
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            let service_name = std::env::var("APP_NAME").unwrap_or("Neodustry".into());

            Resource::builder().with_service_name(service_name).build()
        })
        .clone()
}

pub fn init_logs() -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_http()
        .build()
        .expect("Failed to create log exporter");

    SdkLoggerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .build()
}

fn init_traces() -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_http()
        .build()
        .expect("Failed to create trace exporter");

    SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    let exporter = MetricExporter::builder()
        .with_http()
        .build()
        .expect("Failed to create metric exporter");

    SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(get_resource())
        .build()
}

fn init_pyroscope() -> anyhow::Result<()> {
    let service_name = std::env::var("APP_NAME").unwrap_or("Neodustry".into());

    let sample_rate = std::env::var("PPROF_SAMPLE_RATE")
        .map(|x| x.parse().unwrap_or(100))
        .unwrap_or(100);

    let pprof_back = pprof_backend(
        PprofConfig::new()
            .sample_rate(sample_rate)
            .report_thread_id()
            .report_thread_name(),
    );

    let server = std::env::var("PYROSCOPE_SERVER_ADDRESS")
        .unwrap_or_else(|_| "http://localhost:4040".into());

    let agent = PyroscopeAgent::builder(server, service_name)
        .backend(pprof_back)
        .compression(pyroscope::pyroscope::Compression::GZIP)
        .build()?;

    agent.start()?;

    Ok(())
}
