use super::{MetricEvent, MetricsPrometheus, MovingAverage, Recorder};
use std::time::Duration;
use std::time::Instant;
use std::convert::TryFrom;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use std::sync::{Arc, Mutex};
use bastion::prelude::*;

pub struct MetricsCoordinator;

impl MetricsCoordinator {
    /// No webserver with metrics. The gathering of metrics, discards the data immediately.
    pub fn run_without_metrics_server() -> anyhow::Result<()> {
        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("metrics_worker"))
                    .with_exec(metrics_worker_sink)
            })
        }).expect("Unable to create metrics_worker_sink");
        Ok(())
    }

    /// Runs a webserver with realtime metrics, so bottlenecks can be identified.
    pub fn run_with_metrics_server(listen_on_port: u16, number_of_workers: u64) -> anyhow::Result<()> {
        println!("miner metrics can be downloaded here: http://localhost:{}/metrics", listen_on_port);
    
        let mut registry = <Registry>::default();
        let metrics = MetricsPrometheus::new(&mut registry);
        metrics.number_of_workers.set(number_of_workers);
    
        let registry2: MyRegistry = Arc::new(Mutex::new(registry));
    
        let _ = tokio::spawn(async move {
            let result = webserver_with_metrics(registry2, listen_on_port).await;
            if let Err(error) = result {
                error!("webserver thread failed with error: {:?}", error);
            }
        });

        Bastion::supervisor(|supervisor| {
            supervisor.children(|children| {
                children
                    .with_redundancy(1)
                    .with_distributor(Distributor::named("metrics_worker"))
                    .with_exec(move |ctx: BastionContext| {
                        let metrics_clone = metrics.clone();
                        async move {
                            metrics_worker_webserver(
                                ctx,
                                metrics_clone,
                            ).await
                        }
                    })
            })
        }).expect("Unable to create metrics_worker_webserver");
        Ok(())
    }
}

#[derive(Clone)]
struct State {
    registry: MyRegistry,
}

type MyRegistry = std::sync::Arc<std::sync::Mutex<prometheus_client::registry::Registry<std::boxed::Box<dyn prometheus_client::encoding::text::SendEncodeMetric>>>>;

async fn webserver_with_metrics(registry: MyRegistry, listen_port: u16) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = tide::with_state(State {
        registry: registry,
    });
    app.at("/").get(|_| async { Ok("Hello, world!") });
    app.at("/metrics")
        .get(|req: tide::Request<State>| async move {
            let mut encoded = Vec::new();
            encode(&mut encoded, &req.state().registry.lock().unwrap()).unwrap();
            let response = tide::Response::builder(200)
                .body(encoded)
                .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
                .build();
            Ok(response)
        });
    let server_address = format!("localhost:{}", listen_port);
    app.listen(server_address).await?;
    Ok(())
}

/// Inspect metrics via the Graphana dashboard.
/// 
/// Underneeth Graphana accesses the data via Prometheus.
/// 
/// This function metrics events to Prometheus.
async fn metrics_worker_webserver(ctx: BastionContext, metrics: MetricsPrometheus) -> Result<(), ()> {
    println!("metrics_worker_webserver is ready");
    let mut progress_time = Instant::now();
    let mut miner_iteration_count: u64 = 0;
    let mut moving_average = MovingAverage::new();
    loop {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed > 1000 {
            let elapsed_clamped: u64 = u64::try_from(elapsed).unwrap_or(1000);
            miner_iteration_count *= 1000;
            miner_iteration_count /= elapsed_clamped;

            // Compute average number of iterations over the last second.
            moving_average.insert(miner_iteration_count);
            let weighted_average: u64 = moving_average.average();
            moving_average.rotate();
            metrics.number_of_iteration_now.set(weighted_average);

            progress_time = Instant::now();
            miner_iteration_count = 0;
        }

        let timeout = Duration::from_millis(1000);
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    debug!("metrics_worker_webserver: timeout happened");
                    continue;
                }
                error!("metrics_worker_webserver: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        MessageHandler::new(message)
            .on_tell(|metric_event: MetricEvent, _| {
                if let MetricEvent::General { number_of_iterations, .. } = metric_event {
                    miner_iteration_count += number_of_iterations;
                }
                metrics.record(&metric_event);
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "metrics_worker_webserver {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
    }
}

/// Inspect metrics from commandline.
/// 
/// Prints stats with 1 second interval.
async fn metrics_worker_sink(ctx: BastionContext) -> Result<(), ()> {
    println!("metrics_worker_sink is ready");
    let mut progress_time = Instant::now();
    let mut miner_iteration_count: u64 = 0;
    let mut metric_event_count: u64 = 0;
    loop {
        let elapsed: u128 = progress_time.elapsed().as_millis();
        if elapsed > 1000 {
            let elapsed_clamped: u64 = u64::try_from(elapsed).unwrap_or(1000);
            miner_iteration_count *= 1000;
            miner_iteration_count /= elapsed_clamped;
            metric_event_count *= 1000;
            metric_event_count /= elapsed_clamped;
            
            println!("metrics_worker_sink: metric_events: {} miner_iterations: {}", metric_event_count, miner_iteration_count);
            progress_time = Instant::now();
            miner_iteration_count = 0;
            metric_event_count = 0;
        }

        let timeout = Duration::from_millis(1000);
        let message: SignedMessage = match ctx.try_recv_timeout(timeout).await {
            Ok(message) => message,
            Err(error) => {
                if let ReceiveError::Timeout(_duration) = error {
                    debug!("metrics_worker_sink: timeout happened");
                    continue;
                }
                error!("metrics_worker_sink: Unknown error happened. error: {:?}", error);
                continue;
            }
        };
        MessageHandler::new(message)
            .on_tell(|metric_event: MetricEvent, _| {
                metric_event_count += 1;
                if let MetricEvent::General { number_of_iterations, .. } = metric_event {
                    miner_iteration_count += number_of_iterations;
                }
            })
            .on_fallback(|unknown, _sender_addr| {
                error!(
                    "metrics_worker_sink {}, received an unknown message!:\n{:?}",
                    ctx.current().id(),
                    unknown
                );
            });
    }
}
