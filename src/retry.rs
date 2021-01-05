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
    }
  }

  fn build_command(&self) -> Command {
    let mut cmd = Command::new(&self.command);
    for arg in &self.args {
      cmd.arg(&arg);
    }
    cmd
  }

  fn keep_trying(&mut self) -> bool {
    self.count += 1;
    self.config.max_retries == 0 || self.count < self.config.max_retries
  }

  fn print_command(&self) {
    let mut run = format!("try #{}", self.count + 1);
    if self.config.max_retries != 0 {
      run += &format!("/{}", self.config.max_retries);
    };

    let args = self
      .args
      .iter()
      .map(|a| shlex::quote(&a))
      .collect::<Vec<_>>()
      .join(" ");

    self.log(
      INFO,
      format!("{}: {} {}", &run, shlex::quote(&self.command), args),
    );
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
    match self.config.sleep {
      Some(secs) => Duration::from_secs(secs),
      None => {
        let exp = 2u64.pow(self.count as u32);
        Duration::from_secs(std::cmp::min(exp, self.config.max_sleep))
      }
    }
  }

  pub fn retry(&mut self) {
    loop {
      self.print_command();

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

      let msg = match rc.code() {
        Some(code) => format!("unexpected exit code: {}", code),
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
  }
}
