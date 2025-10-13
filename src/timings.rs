use std::future::Future;

#[cfg(feature = "timings")]
use std::time::Instant;

#[cfg(feature = "timings")]
pub(crate) fn time_block<T, F>(label: &'static str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    tracing::info!(target: "timings", %label, elapsed_ms = elapsed.as_secs_f64() * 1e3);
    value
}

#[cfg(not(feature = "timings"))]
pub(crate) fn time_block<T, F>(_: &'static str, f: F) -> T
where
    F: FnOnce() -> T,
{
    f()
}

#[cfg(feature = "timings")]
pub(crate) async fn time_async_block<T, Fut>(label: &'static str, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    let start = Instant::now();
    let output = fut.await;
    let elapsed = start.elapsed();
    tracing::info!(target: "timings", %label, elapsed_ms = elapsed.as_secs_f64() * 1e3);
    output
}

#[cfg(not(feature = "timings"))]
pub(crate) async fn time_async_block<T, Fut>(_: &'static str, fut: Fut) -> T
where
    Fut: Future<Output = T>,
{
    fut.await
}
