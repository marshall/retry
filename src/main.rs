use std::process::Command;
use std::time::{self, Duration};
use std::{env, thread};

use getopts::{Options, ParsingStyle};

pub mod built_info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

const ERROR: i32 = 0;
const INFO: i32 = 1;
const DEBUG: i32 = 2;

struct Opts {
  max_retries: u64,
  sleep: Option<u64>,
  max_sleep: u64,
  log_level: i32,
  retry_on_success: bool,
  command: Vec<String>,
}

struct Retry {
  opts: Opts,
  count: u64,
  command: String,
  args: Vec<String>,
  start: time::SystemTime,
}

impl Retry {
  pub fn new(opts: Opts) -> Retry {
    let (command, args) = opts.command.split_first().expect("No command provided");

    Retry {
      count: 0,
      command: command.into(),
      args: args.into(),
      start: time::SystemTime::now(),
      /*sleep_strategy: if opts.sleep.is_none() || opts.sleep.unwrap() == 0 {
        Rc::new(RefCell::new(sleep::ExponentialSleep::new()))
      } else {
        Rc::new(RefCell::new(sleep::ConstantSleep::new(opts.sleep.unwrap())))
      },*/
      opts,
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
    self.opts.max_retries == 0 || self.count < self.opts.max_retries
  }

  fn print_command(&self) {
    let mut run = format!("try #{}", self.count + 1);
    if self.opts.max_retries != 0 {
      run += &format!("/{}", self.opts.max_retries);
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
    if self.opts.log_level < level {
      return;
    }

    let time = match time::SystemTime::now().duration_since(self.start) {
      Ok(n) => format!("{:.3}", n.as_secs_f32()),
      Err(_) => "??".to_string(),
    };

    println!("[retry][+{}] {}", time, &msg);
  }

  fn sleep_duration(&self) -> time::Duration {
    match self.opts.sleep {
      Some(secs) => time::Duration::from_secs(secs),
      None => {
        let exp = 2u64.pow(self.count as u32);
        Duration::from_secs(std::cmp::min(exp, self.opts.max_sleep))
      }
    }
  }

  fn retry(&mut self) {
    loop {
      self.print_command();

      let mut cmd = self.build_command();
      let mut child = cmd.spawn().expect("Failed to execute command");
      let rc = child.wait().expect("Failed to wait on command");

      let mut should_retry = !rc.success();
      if self.opts.retry_on_success {
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

    let total_duration = match time::SystemTime::now().duration_since(self.start) {
      Ok(n) => format!("{:.3}", n.as_secs_f32()),
      Err(_) => "??".to_string(),
    };

    self.log(DEBUG, format!("total duration {}s", total_duration));
  }
}

fn version_info() -> String {
  let time = built_info::BUILT_TIME_UTC;
  let mut build_type = built_info::GIT_COMMIT_HASH
    .map(|hash| format!("{}, ", &hash[..8]))
    .unwrap_or("".to_string());

  if built_info::GIT_DIRTY == Some(true) {
    build_type += " (dirty)";
  }

  format!(
    "{} v{} ({}built {})",
    built_info::PKG_NAME,
    built_info::PKG_VERSION,
    build_type,
    &time[..time.len() - 6],
  )
}

fn print_usage(program: &str, opts: &Options) {
  let brief = format!(
    "{}\nUsage: {} [options] [--] cmd [args..]",
    version_info(),
    program,
  );

  print!("{}\n", opts.usage(&brief));
}

fn parse() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.parsing_style(ParsingStyle::StopAtFirstFree);
  opts.optflag("h", "help", "Print this help menu");
  opts.optopt(
    "m",
    "max-sleep",
    "Max sleep in seconds between retries when using exponential backoff. default=3600 (1 hour)",
    "n",
  );
  opts.optopt(
    "n",
    "max-retries",
    "Max number of retries. Set to 0 for unlimited retries",
    "n",
  );
  opts.optflag("q", "quiet", "Don't log anything");
  opts.optopt(
    "s",
    "sleep",
    "Sleep n seconds between retries. Overrides default exponential backoff.",
    "n",
  );
  opts.optflag("v", "verbose", "More verbose logging");
  opts.optflag("V", "version", "Print version information");
  opts.optflag(
    "x",
    "retry-on-success",
    "Retry when a program has an exit code of 0",
  );

  let matches = opts
    .parse(&args[1..])
    .ok()
    .ok_or_else(|| "Couldn't parse args")?;

  if matches.opt_present("h") {
    print_usage(&program, &opts);
    return Ok(());
  }

  if matches.opt_present("V") {
    println!("{}", version_info());
    return Ok(());
  }

  let max_retries = matches.opt_get_default("n", 5).map_err(|_| {
    print_usage(&program, &opts);
    "Invalid max-retries, must be a number".to_string()
  })?;

  let sleep = matches.opt_get::<u64>("s").map_err(|_| {
    print_usage(&program, &opts);
    "Invalid sleep, must be number of seconds".to_string()
  })?;

  let max_sleep = matches.opt_get_default("m", 3600).map_err(|_| {
    print_usage(&program, &opts);
    "Invalid max-sleep, must be number of seconds.".to_string()
  })?;

  let log_level = if matches.opt_present("v") {
    DEBUG
  } else if matches.opt_present("q") {
    ERROR
  } else {
    INFO
  };

  let retry_on_success = matches.opt_present("x");
  let command = matches.free;
  if command.len() == 0 {
    print_usage(&program, &opts);
    return Err("No command provided".to_string());
  }

  let opts = Opts {
    max_retries,
    sleep,
    max_sleep,
    log_level,
    retry_on_success,
    command,
  };

  Retry::new(opts).retry();
  Ok(())
}

fn main() {
  if let Err(e) = parse() {
    println!("Error: {}", e);
  }
}
