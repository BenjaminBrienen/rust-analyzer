//! Discovery of `cargo` & `rustc` executables.

use std::{env, ffi::OsStr, iter, process::Command};

use paths::{AbsPath, AbsPathBuf};

#[derive(Copy, Clone)]
pub enum Tool {
    Cargo,
    Rustc,
    Rustup,
    Rustfmt,
}

impl Tool {
    pub fn proxy(self) -> Option<AbsPathBuf> {
        cargo_proxy(self.name())
    }

    /// Return a `PathBuf` to use for the given executable.
    ///
    /// The current implementation checks three places for an executable to use:
    /// 1) `$CARGO_HOME/bin/<executable_name>`
    ///    where $CARGO_HOME defaults to ~/.cargo (see <https://doc.rust-lang.org/cargo/guide/cargo-home.html>)
    ///    example: for cargo, this tries $CARGO_HOME/bin/cargo, or ~/.cargo/bin/cargo if $CARGO_HOME is unset.
    ///    It seems that this is a reasonable place to try for cargo, rustc, and rustup
    /// 2) Appropriate environment variable (erroring if this is set but not a usable executable)
    ///    example: for cargo, this checks $CARGO environment variable; for rustc, $RUSTC; etc
    /// 3) $PATH/`<executable_name>`
    ///    example: for cargo, this tries all paths in $PATH with appended `cargo`, returning the
    ///    first that exists
    /// 4) If all else fails, we just try to use the executable name directly
    pub fn prefer_proxy(self) -> AbsPathBuf {
        invoke(&[cargo_proxy, lookup_as_env_var, lookup_in_path], self.name())
    }

    /// Return a `PathBuf` to use for the given executable.
    ///
    /// The current implementation checks three places for an executable to use:
    /// 1) Appropriate environment variable (erroring if this is set but not a usable executable)
    ///    example: for cargo, this checks $CARGO environment variable; for rustc, $RUSTC; etc
    /// 2) $PATH/`<executable_name>`
    ///    example: for cargo, this tries all paths in $PATH with appended `cargo`, returning the
    ///    first that exists
    /// 3) `$CARGO_HOME/bin/<executable_name>`
    ///    where $CARGO_HOME defaults to ~/.cargo (see <https://doc.rust-lang.org/cargo/guide/cargo-home.html>)
    ///    example: for cargo, this tries $CARGO_HOME/bin/cargo, or ~/.cargo/bin/cargo if $CARGO_HOME is unset.
    ///    It seems that this is a reasonable place to try for cargo, rustc, and rustup
    /// 4) If all else fails, we just try to use the executable name directly
    pub fn path(self) -> AbsPathBuf {
        invoke(&[lookup_as_env_var, lookup_in_path, cargo_proxy], self.name())
    }

    pub fn path_in(self, path: &AbsPath) -> Option<AbsPathBuf> {
        probe_for_binary(path.join(self.name()))
    }

    pub fn name(self) -> &'static str {
        match self {
            Tool::Cargo => "cargo",
            Tool::Rustc => "rustc",
            Tool::Rustup => "rustup",
            Tool::Rustfmt => "rustfmt",
        }
    }
}

// Prevent rustup from automatically installing toolchains, see https://github.com/rust-lang/rust-analyzer/issues/20719.
pub const NO_RUSTUP_AUTO_INSTALL_ENV: (&str, &str) = ("RUSTUP_AUTO_INSTALL", "0");

#[allow(clippy::disallowed_types, reason = "generic parameters allow for FxHashMap and AbsPath")]
pub fn command<H>(
    cmd: impl AsRef<OsStr>,
    working_directory: impl AsRef<std::path::Path>,
    extra_env: &std::collections::HashMap<String, Option<String>, H>,
) -> Command {
    #[expect(clippy::disallowed_methods, reason = "we are `toolchain::command`")]
    let mut cmd = Command::new(cmd);
    cmd.current_dir(working_directory.as_ref());
    cmd.env(NO_RUSTUP_AUTO_INSTALL_ENV.0, NO_RUSTUP_AUTO_INSTALL_ENV.1);
    for env in extra_env {
        match env {
            (key, Some(val)) => cmd.env(key, val),
            (key, None) => cmd.env_remove(key),
        };
    }
    cmd
}

fn invoke(list: &[fn(&str) -> Option<AbsPathBuf>], executable: &str) -> AbsPathBuf {
    list.iter()
        .find_map(|it| it(executable))
        .unwrap_or_else(|| AbsPathBuf::make_absolute(&executable))
}

/// Looks up the binary as its SCREAMINGUPPERCASE in the env variables.
fn lookup_as_env_var(executable_name: &str) -> Option<AbsPathBuf> {
    env::var_os(executable_name.to_ascii_uppercase()).map(|path| AbsPathBuf::make_absolute(&path))
}

/// Looks up the binary in the cargo home directory if it exists.
fn cargo_proxy(executable_name: &str) -> Option<AbsPathBuf> {
    let mut path = get_cargo_home()?;
    path.push("bin");
    path.push(executable_name);
    probe_for_binary(path)
}

fn get_cargo_home() -> Option<AbsPathBuf> {
    if let Some(path) = env::var_os("CARGO_HOME") {
        return Some(AbsPathBuf::make_absolute(&path));
    }

    if let Some(mut path) = home::home_dir() {
        path.push(".cargo");
        return Some(AbsPathBuf::make_absolute(&path));
    }

    None
}

fn lookup_in_path(exec: &str) -> Option<AbsPathBuf> {
    let paths = env::var_os("PATH").unwrap_or_default();
    env::split_paths(&paths)
        .filter_map(|path| AbsPathBuf::try_make_absolute(&path).ok()) // filter out non-utf8 PATH paths
        .map(|path| path.join(exec))
        .find_map(probe_for_binary)
}

pub fn probe_for_binary(path: AbsPathBuf) -> Option<AbsPathBuf> {
    let with_extension = match env::consts::EXE_EXTENSION {
        "" => None,
        it => Some(path.with_extension(it)),
    };
    iter::once(path).chain(with_extension).find(|it| it.is_file())
}
