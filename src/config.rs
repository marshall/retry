pub struct RetryConfig {
  pub max_retries: u64,
  pub sleep: Option<u64>,
  pub max_sleep: u64,
  pub log_level: i32,
  pub quiet: bool,
  pub retry_on_success: bool,
  pub command: Vec<String>,
}

impl Default for RetryConfig {
  fn default() -> Self {
    RetryConfig {
      max_retries: 5,
      sleep: None,
      max_sleep: 3600,
      log_level: 1,
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

  pub fn max_retries(&mut self, n: u64) -> &mut Self {
    self.max_retries = n;
    self
  }

  pub fn sleep(&mut self, sleep: Option<u64>) -> &mut Self {
    self.sleep = sleep;
    self
  }

  pub fn max_sleep(&mut self, max_sleep: u64) -> &mut Self {
    self.max_sleep = max_sleep;
    self
  }

  pub fn log_level(&mut self, log_level: i32) -> &mut Self {
    self.log_level = log_level;
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
