//! Shared testing utilities for gho.

use assert_cmd::Command;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Testing harness providing an isolated HOME/workspace pair for CLI and SDK exercises.
#[allow(dead_code)]
pub struct TestContext {
    root: TempDir,
    work_dir: PathBuf,
    original_home: Option<OsString>,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new isolated environment and point `HOME` to it so the CLI uses local storage.
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp directory for tests");
        let work_dir = root.path().join("work");
        fs::create_dir_all(&work_dir).expect("Failed to create test work directory");

        let original_home = env::var_os("HOME");
        unsafe {
            env::set_var("HOME", root.path());
        }

        Self { root, work_dir, original_home }
    }

    /// Absolute path to the emulated `$HOME` directory.
    pub fn home(&self) -> &Path {
        self.root.path()
    }

    /// Path to the workspace directory used for CLI invocations.
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Path to the gho config directory.
    pub fn config_dir(&self) -> PathBuf {
        self.home().join(".config").join("gho")
    }

    /// Path to the accounts.json file.
    pub fn accounts_path(&self) -> PathBuf {
        self.config_dir().join("accounts.json")
    }

    /// Build a command for invoking the compiled `gho` binary within the default workspace.
    pub fn cli(&self) -> Command {
        self.cli_in(self.work_dir())
    }

    /// Build a command for invoking the compiled `gho` binary within a custom directory.
    pub fn cli_in<P: AsRef<Path>>(&self, dir: P) -> Command {
        let mut cmd = Command::cargo_bin("gho").expect("Failed to locate gho binary");
        cmd.current_dir(dir.as_ref()).env("HOME", self.home());
        cmd
    }

    /// Write accounts.json with given content.
    pub fn write_accounts(&self, content: &str) {
        fs::create_dir_all(self.config_dir()).expect("Failed to create config dir");
        fs::write(self.accounts_path(), content).expect("Failed to write accounts.json");
    }

    /// Read accounts.json content.
    pub fn read_accounts(&self) -> String {
        fs::read_to_string(self.accounts_path()).unwrap_or_default()
    }

    /// Execute a closure after temporarily switching into the provided directory.
    pub fn with_dir<F, R, P>(&self, dir: P, action: F) -> R
    where
        F: FnOnce() -> R,
        P: AsRef<Path>,
    {
        let original = env::current_dir().expect("Failed to capture current dir");
        env::set_current_dir(dir.as_ref()).expect("Failed to switch current dir");
        let result = action();
        env::set_current_dir(original).expect("Failed to restore current dir");
        result
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        match &self.original_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }
    }
}
