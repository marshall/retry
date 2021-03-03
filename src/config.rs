use crate::retry::{DEBUG, INFO};

pub struct RetryConfig {
  pub max_tries: u64,
  pub sleep: u64,
  pub backoff: bool,
  pub max_backoff: u64,
  pub log_level: i32,
  pub quiet: bool,
  pub retry_on_success: bool,
  pub command: Vec<String>,
}

impl Default for RetryConfig {
  fn default() -> Self {
    RetryConfig {
      max_tries: 10,
      sleep: 5,
      backoff: false,
      max_backoff: 60,
      log_level: INFO,
      quiet: false,
      retry_on_success: false,
      command: vec![],
    }
  }
}

impl RetryConfig {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn max_tries(&mut self, n: u64) -> &mut Self {
    self.max_tries = n;
    self
  }

  pub fn sleep(&mut self, sleep: u64) -> &mut Self {
    self.sleep = sleep;
    self
  }

  pub fn backoff(&mut self, backoff: bool) -> &mut Self {
    self.backoff = backoff;
    self
  }

  pub fn max_backoff(&mut self, max_backoff: u64) -> &mut Self {
    self.max_backoff = max_backoff;
    self
  }

  pub fn log_level(&mut self, log_level: i32) -> &mut Self {
    self.log_level = log_level;
    self
  }

  pub fn verbose(&mut self, verbose: bool) -> &mut Self {
    self.log_level = if verbose { DEBUG } else { INFO };
    self
  }

  pub fn quiet(&mut self, quiet: bool) -> &mut Self {
    self.quiet = quiet;
    self
  }

  pub fn retry_on_success(&mut self, retry_on_success: bool) -> &mut Self {
    self.retry_on_success = retry_on_success;
    self
  }

  pub fn command(&mut self, command: Vec<String>) -> &mut Self {
    self.command = command;
    self
  }
}
