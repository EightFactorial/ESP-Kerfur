#![allow(dead_code, reason = "These functions may or may not be used")]

/// Retries a function up to N times until it succeeds.
#[must_use]
pub(crate) fn retry<const N: usize, F: FnMut() -> Result<T, E>, T, E>(mut f: F) -> Result<T, E> {
    let mut result = f();
    for _ in 1..N {
        if result.is_ok() {
            break;
        }
        result = f();
    }
    result
}

/// Retries a function up to N times until it succeeds.
#[must_use]
pub(crate) async fn retry_async<const N: usize, F: AsyncFnMut() -> Result<T, E>, T, E>(
    mut f: F,
) -> Result<T, E> {
    let mut result = f().await;
    for _ in 1..N {
        if result.is_ok() {
            break;
        }
        result = f().await;
    }
    result
}
