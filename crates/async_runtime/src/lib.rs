use core::future::Future;

#[cfg(target_arch = "wasm32")]
pub fn call_async<T>(future: impl Future<Output = T>) -> impl Future<Output = T> + Send {
    send_wrapper::SendWrapper::new(future)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn call_async<F>(future: F) -> F
where
    F: Future + Send,
{
    future
}
