use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime};

use crate::config::RetryConfig;

pub const INFO: i32 = 0;
pub const DEBUG: i32 = 1;

pub struct Retry {
  config: RetryConfig,
  count: u64,
  command: String,
  args: Vec<String>,
  start: SystemTime,
  prev_exit_code: Option<i32>,
}

impl Retry {
  pub fn new(config: RetryConfig) -> Retry {
    let (command, args) = config.command.split_first().expect("No command provided");

    Retry {
      count: 0,
      command: command.into(),
      args: args.into(),
      start: SystemTime::now(),
      config,
      prev_exit_code: None,
    }
  }

  fn build_command(&self) -> Command {
    let mut cmd = Command::new(&self.command);
    let prev_sleep = self.sleep_duration().as_secs();
    let next_sleep = self.sleep_duration_from_count(self.count + 1).as_secs();

    cmd
      .args(&self.args)
      .env("RETRY_TRY", (self.count + 1).to_string())
      .env("RETRY_MAX", self.config.max_tries.to_string())
      .env("RETRY_NEXT_SLEEP", next_sleep.to_string());

    if self.count > 0 {
      cmd.env("RETRY_PREV_SLEEP", prev_sleep.to_string());
    }

    if let Some(prev_exit_code) = self.prev_exit_code {
      cmd.env("RETRY_PREV_EXIT_CODE", prev_exit_code.to_string());
    }

    cmd
  }

  fn keep_trying(&mut self) -> bool {
    self.count += 1;
    self.config.max_tries == 0 || self.count < self.config.max_tries
  }

  fn log_run(&self) {
    let run = format!(
      "try #{}{}: {} {}",
      self.count + 1,
      if self.config.max_tries != 0 {
        format!("/{}", self.config.max_tries)
      } else {
        "".to_string()
      },
      &shlex::quote(&self.command),
      shlex::join(&self.args)
    );

    self.log(INFO, run);
  }

  fn log(&self, level: i32, msg: String) {
    if self.config.quiet || self.config.log_level < level {
      return;
    }

    let time = match SystemTime::now().duration_since(self.start) {
      Ok(n) => format!("{:.3}", n.as_secs_f32()),
      Err(_) => "??".to_string(),
    };

    println!("[retry][+{}] {}", time, &msg);
  }

  fn sleep_duration(&self) -> Duration {
    self.sleep_duration_from_count(self.count)
  }

  fn sleep_duration_from_count(&self, count: u64) -> Duration {
    if self.config.backoff {
      let exp = 2u64.pow(count as u32);
      Duration::from_secs(std::cmp::min(exp, self.config.max_backoff))
    } else {
      Duration::from_secs(self.config.sleep)
    }
  }

  pub fn retry(&mut self) -> Option<i32> {
    loop {
      self.log_run();

      let mut cmd = self.build_command();
      let mut child = cmd.spawn().expect("Failed to execute command");
      let rc = child.wait().expect("Failed to wait on command");

      let mut should_retry = !rc.success();
      if self.config.retry_on_success {
        should_retry = !should_retry;
      }

      if !should_retry || !self.keep_trying() {
        break;
      }

      self.prev_exit_code = rc.code();
      let msg = match self.prev_exit_code {
        Some(code) => {
          format!("unexpected exit code: {}", code)
        }
        None => "process terminated by signal".to_string(),
      };

      let duration = self.sleep_duration();
      self.log(DEBUG, format!("{}, sleeping {}s", msg, duration.as_secs()));

      thread::sleep(duration);
    }

    let total_duration = match SystemTime::now().duration_since(self.start) {
      Ok(n) => format!("{:.3}", n.as_secs_f32()),
      Err(_) => "??".to_string(),
    };

    self.log(DEBUG, format!("total duration {}s", total_duration));
    self.prev_exit_code
  }
}
