use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use getopts::Options;

fn main() -> Result<(), Box<dyn Error>> {
  let args: Vec<String> = env::args().collect();
  let mut opts = Options::new();
  opts.reqopt("s", "state-dir", "dir to write state", "FILE");
  opts.optopt("x", "exit-code", "Exit with CODE", "CODE");

  let matches = opts.parse(&args[1..]).ok().ok_or("Invalid arguments")?;
  let state_dir = PathBuf::from(matches.opt_str("s").ok_or("No state dir")?);
  let run_count_file = state_dir.join("run_count");

  let mut run_count: u64 = if !run_count_file.exists() {
    0
  } else {
    let contents =
      fs::read_to_string(&run_count_file).expect("Something went wrong reading the file");
    contents.parse().unwrap_or(0)
  };

  run_count += 1;

  for var in &[
    "RETRY_TRY",
    "RETRY_MAX",
    "RETRY_PREV_EXIT_CODE",
    "RETRY_PREV_SLEEP",
    "RETRY_NEXT_SLEEP",
  ] {
    fs::write(
      state_dir.join(var),
      format!("{}", env::var(var).unwrap_or("None".to_string())),
    )?;
  }

  fs::write(run_count_file, format!("{}", run_count))?;

  println!("{}", run_count);

  std::process::exit(matches.opt_get_default("x", 0).unwrap_or(0));
}
