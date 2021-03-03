use std::env;
use std::error::Error;

use getopts::{Matches, Options, ParsingStyle};

#[allow(dead_code)]
pub mod built_info {
  include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub mod config;
pub mod retry;

use config::RetryConfig;
use retry::Retry;

struct RetryCommand {
  opts: Options,
  matches: Matches,
}

impl RetryCommand {
  fn from_args() -> Result<Self, String> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.parsing_style(ParsingStyle::StopAtFirstFree);
    opts.optflag(
      "b",
      "backoff",
      "Sleep times between each try will increase exponentially (up to max-sleep)",
    );
    opts.optflag("h", "help", "Print this help menu");
    opts.optopt(
      "m",
      "max-backoff",
      "Max sleep in seconds between tries when using exponential backoff. default=60 (1 minute)",
      "n",
    );
    opts.optopt(
      "n",
      "max-tries",
      "Max number of tries. Set to 0 for unlimited tries. default=10",
      "n",
    );
    opts.optflag("q", "quiet", "Don't log anything");
    opts.optopt(
      "s",
      "sleep",
      "Sleep n seconds between tries. default=5",
      "n",
    );
    opts.optflag("v", "verbose", "More verbose logging");
    opts.optflag("V", "version", "Print version information");
    opts.optflag(
      "x",
      "retry-on-success",
      "Retry when 'cmd' has an exit code of 0",
    );

    let matches = opts
      .parse(&args[1..])
      .ok()
      .ok_or_else(|| "Couldn't parse args")?;

    Ok(Self { opts, matches })
  }

  pub fn run(&self) -> Result<Option<i32>, String> {
    let mut config = RetryConfig::new();
    config
      .quiet(self.matches.opt_present("q"))
      .verbose(self.matches.opt_present("v"))
      .retry_on_success(self.matches.opt_present("x"))
      .backoff(self.matches.opt_present("b"));

    if self.matches.opt_present("h") {
      self.print_usage();
      return Ok(Some(0));
    }

    if self.matches.opt_present("V") {
      println!("{}", self.version_info());
      return Ok(Some(0));
    }

    config.max_tries(
      self
        .matches
        .opt_get_default("n", config.max_tries)
        .map_err(|_| {
          self.print_usage();
          "Invalid max-tries, must be a number".to_string()
        })?,
    );

    config.sleep(
      self
        .matches
        .opt_get_default("s", config.sleep)
        .map_err(|_| {
          self.print_usage();
          "Invalid sleep, must be number of seconds".to_string()
        })?,
    );

    config.max_backoff(
      self
        .matches
        .opt_get_default("m", config.max_backoff)
        .map_err(|_| {
          self.print_usage();
          "Invalid max-backoff, must be number of seconds.".to_string()
        })?,
    );

    let command = self.matches.free.clone();
    if command.len() == 0 {
      self.print_usage();
      return Err("No command provided".to_string());
    }

    config.command(command);
    Ok(Retry::new(config).retry())
  }

  fn version_info(&self) -> String {
    if self.matches.opt_present("q") {
      return format!("{} v{}", built_info::PKG_NAME, built_info::PKG_VERSION);
    }

    let time = built_info::BUILT_TIME_UTC;
    let mut build_type = built_info::GIT_COMMIT_HASH
      .map(|hash| format!("{}", &hash[..8]))
      .unwrap_or("".to_string());

    if built_info::GIT_DIRTY == Some(true) {
      build_type += " (dirty)";
    }

    format!(
      "{} v{} - {} on {}",
      built_info::PKG_NAME,
      built_info::PKG_VERSION,
      build_type,
      &time[..time.len() - 6]
    )
  }

  fn print_usage(&self) {
    let brief = format!(
      "{}\nUsage: {} [options] [--] cmd [args..]",
      self.version_info(),
      built_info::PKG_NAME,
    );

    print!("{}\n", self.opts.usage(&brief));
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  if let Some(exit_code) = RetryCommand::from_args()?.run()? {
    std::process::exit(exit_code);
  }

  Ok(())
}
