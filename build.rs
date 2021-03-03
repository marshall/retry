use std::{env, path::Path};

fn main() {
  let mut opts = built::Options::default();
  opts
    .set_cfg(false)
    .set_compiler(false)
    .set_dependencies(false)
    .set_features(false)
    .set_ci(true)
    .set_env(true)
    .set_git(true)
    .set_time(true);

  let src = env::var("CARGO_MANIFEST_DIR").unwrap();
  let dst = Path::new(&env::var("OUT_DIR").unwrap()).join("built.rs");
  built::write_built_file_with_opts(&opts, src.as_ref(), &dst)
    .expect("Failed to acquire build-time information");
}
