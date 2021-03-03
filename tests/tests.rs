use assert_fs::prelude::*;
use predicates::str::ends_with;

mod retry_test;
use retry_test::*;

#[test]
fn test_default() -> TestResult {
  RetryCommand::new()?
    .command(&["echo", "-n", "test"])
    .assert()
    .success()
    .stdout(ends_with("test"));

  Ok(())
}

#[test]
fn test_retry_on_success() -> TestResult {
  RetryCommand::new()?
    .arg("--retry-on-success")
    .command(&["/bin/sh", "-c", "test \"$RETRY_TRY\" = \"1\""])
    .assert()
    .success();

  Ok(())
}

#[test]
fn test_max_tries() -> TestResult {
  let temp = assert_fs::TempDir::new()?;
  RetryCommand::new()?
    .arg("--max-tries=3")
    .arg("--sleep=1")
    .with_mock_cmd(|mock| mock.state_dir(temp.path()).exit_code(1))
    .assert()
    .failure()
    .stdout(ends_with("3\n"));

  temp.child("run_count").assert("3");
  Ok(())
}

#[test]
fn test_env_defaults() -> TestResult {
  let temp = assert_fs::TempDir::new()?;
  RetryCommand::new()?
    .with_mock_cmd(|mock| mock.state_dir(temp.path()))
    .assert()
    .success();

  temp.child("RETRY_TRY").assert("1");
  temp.child("RETRY_MAX").assert("10");
  temp.child("RETRY_PREV_SLEEP").assert("None");
  temp.child("RETRY_NEXT_SLEEP").assert("5");
  temp.child("RETRY_PREV_EXIT_CODE").assert("None");

  Ok(())
}

#[test]
fn test_backoff() -> TestResult {
  let temp = assert_fs::TempDir::new()?;
  RetryCommand::new()?
    .arg("--backoff")
    .arg("--max-tries=3")
    .with_mock_cmd(|mock| mock.state_dir(temp.path()).exit_code(1))
    .assert()
    .failure();

  temp.child("RETRY_TRY").assert("3");
  temp.child("RETRY_MAX").assert("3");
  temp.child("RETRY_PREV_SLEEP").assert("4");
  temp.child("RETRY_NEXT_SLEEP").assert("8");

  Ok(())
}

#[test]
fn test_max_backoff() -> TestResult {
  let temp = assert_fs::TempDir::new()?;
  RetryCommand::new()?
    .arg("--backoff")
    .arg("--max-tries=2")
    .arg("--max-backoff=3")
    .with_mock_cmd(|mock| mock.state_dir(temp.path()).exit_code(1))
    .assert()
    .failure();

  temp.child("RETRY_TRY").assert("2");
  temp.child("RETRY_MAX").assert("2");
  temp.child("RETRY_PREV_SLEEP").assert("2");
  temp.child("RETRY_NEXT_SLEEP").assert("3");

  Ok(())
}

#[test]
fn test_quiet() -> TestResult {
  RetryCommand::new()?
    .arg("--quiet")
    .command(&["echo", "-n", "shhh"])
    .assert()
    .success()
    .stdout("shhh");

  Ok(())
}
