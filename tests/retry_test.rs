use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::path::Path;

use assert_cmd::{assert::Assert, Command};

pub const MOCK_CMD_PATH: &'static str = env!("CARGO_BIN_EXE_retry-mock-cmd");

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub struct RetryCommand {
  retry: Command,
  command: Vec<String>,
}

impl RetryCommand {
  pub fn new() -> TestResult<Self> {
    Ok(RetryCommand {
      retry: Command::cargo_bin("retry")?,
      command: vec![],
    })
  }

  pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
    self.retry.arg(arg);
    self
  }

  pub fn args<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
  {
    self.retry.args(args);
    self
  }

  pub fn command<I, S>(&mut self, args: I) -> &mut Self
  where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
  {
    for arg in args.into_iter() {
      self.command.push(arg.as_ref().to_string());
    }
    self
  }

  pub fn with_mock_cmd(
    &mut self,
    callback: impl Fn(&mut RetryMockCommand) -> &mut RetryMockCommand,
  ) -> &mut Self {
    let mut mock = RetryMockCommand::new();
    callback(&mut mock);
    self.args(mock.build())
  }

  pub fn build(&mut self) -> &mut Command {
    self.retry.arg("--").args(self.command.to_owned());
    &mut self.retry
  }

  pub fn assert(&mut self) -> Assert {
    self.build().assert()
  }
}

pub struct RetryMockCommand {
  state_dir: String,
  exit_code: i32,
}

impl RetryMockCommand {
  pub fn new() -> Self {
    RetryMockCommand {
      state_dir: "".to_string(),
      exit_code: 0,
    }
  }

  pub fn state_dir<T: AsRef<Path>>(&mut self, state_dir: T) -> &mut Self {
    self.state_dir = state_dir.as_ref().to_string_lossy().to_string();
    self
  }

  pub fn exit_code(&mut self, exit_code: i32) -> &mut Self {
    self.exit_code = exit_code;
    self
  }

  pub fn build(&mut self) -> Vec<String> {
    vec![
      MOCK_CMD_PATH.to_string(),
      "--state-dir".to_string(),
      self.state_dir.to_owned(),
      "--exit-code".to_string(),
      self.exit_code.to_string(),
    ]
  }
}
