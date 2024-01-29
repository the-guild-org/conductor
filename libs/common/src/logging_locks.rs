use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::{Mutex, RwLock};
use tracing::info;

pub use tokio::sync::MutexGuard;
pub use tokio::sync::RwLockReadGuard;
pub use tokio::sync::RwLockWriteGuard;

pub struct LoggingMutex<T> {
  inner: Mutex<T>,
  id: String,
}

impl<T> LoggingMutex<T> {
  pub fn new(id: impl Into<String>, value: T) -> Self {
    LoggingMutex {
      inner: Mutex::new(value),
      id: id.into(),
    }
  }

  pub fn lock(&self) -> LoggingMutexGuard<'_, T> {
    info!(
      "Attempting to lock Mutex with ID: \"{}\" at {}:{}",
      self.id,
      file!(),
      line!()
    );
    LoggingMutexGuard {
      guard: Box::pin(self.inner.lock()),
      id: &self.id,
    }
  }
}

pub struct LoggingMutexGuard<'a, T> {
  guard: Pin<Box<dyn Future<Output = MutexGuard<'a, T>> + 'a>>,
  id: &'a str,
}

impl<'a, T> Future for LoggingMutexGuard<'a, T> {
  type Output = MutexGuard<'a, T>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.guard.as_mut().poll(cx)
  }
}

impl<T> Drop for LoggingMutexGuard<'_, T> {
  fn drop(&mut self) {
    info!(
      "Mutex with ID: \"{}\" is unlocked at {}:{}",
      self.id,
      file!(),
      line!()
    );
  }
}

pub struct LoggingRwLock<T> {
  inner: RwLock<T>,
  id: String,
}

impl<T> LoggingRwLock<T> {
  pub fn new(id: impl Into<String>, value: T) -> Self {
    LoggingRwLock {
      inner: RwLock::new(value),
      id: id.into(),
    }
  }

  pub fn read(&self) -> LoggingRwLockReadGuard<'_, T> {
    info!(
      "Attempting to acquire read lock with ID: \"{}\" at {}:{}",
      self.id,
      file!(),
      line!()
    );
    LoggingRwLockReadGuard {
      guard: Box::pin(self.inner.read()),
      id: &self.id,
    }
  }

  pub fn write<'a>(&'a self) -> LoggingRwLockWriteGuard<'a, T> {
    info!(
      "Attempting to acquire write lock with ID: \"{}\" at {}:{}",
      self.id,
      file!(),
      line!()
    );
    LoggingRwLockWriteGuard {
      guard: Box::pin(self.inner.write()),
      id: &self.id,
    }
  }
}

pub struct LoggingRwLockReadGuard<'a, T> {
  guard: Pin<Box<dyn Future<Output = RwLockReadGuard<'a, T>> + 'a>>,
  id: &'a str,
}

impl<'a, T> Future for LoggingRwLockReadGuard<'a, T> {
  type Output = RwLockReadGuard<'a, T>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.guard.as_mut().poll(cx)
  }
}

impl<T> Drop for LoggingRwLockReadGuard<'_, T> {
  fn drop(&mut self) {
    info!(
      "Read lock with ID: {} is released at {}:{}",
      self.id,
      file!(),
      line!()
    );
  }
}

pub struct LoggingRwLockWriteGuard<'a, T> {
  guard: Pin<Box<dyn Future<Output = RwLockWriteGuard<'a, T>> + 'a>>,
  id: &'a str,
}

impl<'a, T> Future for LoggingRwLockWriteGuard<'a, T> {
  type Output = RwLockWriteGuard<'a, T>;

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.guard.as_mut().poll(cx)
  }
}

impl<T> Drop for LoggingRwLockWriteGuard<'_, T> {
  fn drop(&mut self) {
    info!(
      "Write lock with ID: {} is released at {}:{}",
      self.id,
      file!(),
      line!()
    );
  }
}
