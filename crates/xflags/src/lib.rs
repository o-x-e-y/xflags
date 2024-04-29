//! `xflags` provides a procedural macro for parsing command line arguments.
//!
//! It is intended for use in development tools, so it emphasizes fast compile
//! times and convenience at the expense of features.
//!
//! Rough decision tree for picking an argument parsing library:
//!
//! * if you need all of the features and don't care about minimalism, use
//!   [clap](https://github.com/clap-rs/clap)
//! * if you want to be maximally minimal, need only basic features (eg, no help
//!   generation), and want to be pedantically correct, use
//!   [lexopt](https://github.com/blyxxyz/lexopt)
//! * if you want to get things done fast (eg, you want auto help, but not at
//!   the cost of waiting for syn to compile), consider this crate.
//!
//! The secret sauce of xflags is that it is the opposite of a derive macro.
//! Rather than generating a command line grammar from a Rust struct, `xflags`
//! generates Rust structs based on input grammar. The grammar definition is
//! both shorter and simpler to write, and is lighter on compile times.
//!
//! Here's a complete example of `parse_or_exit!` macro which parses arguments
//! into an "anonymous" struct:
//!
//! ```no_run
//! use std::path::PathBuf;
//!
//! fn main() {
//!     let flags = xflags::parse_or_exit! {
//!         /// Remove directories and their contents recursively.
//!         optional -r,--recursive
//!         /// File or directory to remove
//!         required path: PathBuf
//!     };
//!
//!     println!(
//!         "removing {}{}",
//!         flags.path.display(),
//!         if flags.recursive { "recursively" } else { "" },
//!     )
//! }
//! ```
//!
//! The above program, when run with `--help` argument, generates the following
//! help:
//!
//! ```text
//! Usage:  <path> [-r] [-h]
//! Arguments:
//!   <path>               File or directory to remove
//!
//! Options:
//!   -r, --recursive      Remove directories and their contents recursively.
//!   -h, --help           Prints help
//!
//! Commands:
//!   help                 Print this message or the help of the given subcommand(s)
//! ```
//!
//! For larger programs, you'd typically want to use `xflags!` macro, which
//! generates _named_ structs for you. Unlike a typical macro, `xflags` writes
//! generated code into the source file, to make it easy to understand the rust
//! types at a glance.
//!
//! ```
//! mod flags {
//!     use std::path::PathBuf;
//!
//!     xflags::xflags! {
//!         src "./examples/basic.rs"
//!
//!         cmd my-command {
//!             required path: PathBuf
//!             optional -v, --verbose
//!         }
//!     }
//!
//!     // generated start
//!     // The following code is generated by `xflags` macro.
//!     // Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
//!     #[derive(Debug)]
//!     pub struct MyCommand {
//!         pub path: PathBuf,
//!         pub verbose: bool,
//!     }
//!
//!     impl MyCommand {
//!         pub fn from_env_or_exit() -> Self {
//!             Self::from_env_or_exit_()
//!         }
//!         pub fn from_env() -> xflags::Result<Self> {
//!             Self::from_env_()
//!         }
//!         pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
//!             Self::from_vec_(args)
//!         }
//!     }
//!    // generated end
//! }
//!
//! fn main() {
//!     let flags = flags::MyCommand::from_env();
//!     println!("{:#?}", flags);
//! }
//! ```
//!
//! If you'd rather use a typical proc-macro which generates hidden code, just
//! omit the src attribute.
//!
//! xflags correctly handles non-utf8 arguments.
//!
//! ## Syntax Reference
//!
//! The `xflags!` macro uses **cmd** keyword to introduce a command or
//! subcommand that accepts positional arguments and switches.
//!
//! ```
//! xflags::xflags! {
//!     cmd command-name { }
//! }
//! ```
//!
//! Switches are specified inside the curly braces. Long names (`--switch`) are
//! mandatory, short names (`-s`) are optional. Each switch can be **optional**,
//! **required**, or **repeated**. Dashes are allowed in switch names.
//!
//! ```
//! xflags::xflags! {
//!     cmd switches {
//!         optional -q,--quiet
//!         required --pass-me
//!         repeated --verbose
//!     }
//! }
//! ```
//!
//! Switches can also take values. If the value type is `OsString` or `PathBuf`,
//! it is created directly from the underlying argument. Otherwise, `FromStr` is
//! used for parsing
//!
//! ```
//! use std::{path::PathBuf, ffi::OsString};
//!
//! xflags::xflags! {
//!     cmd switches-with-values {
//!         optional --config path: PathBuf
//!         repeated --data val: OsString
//!         optional -j, --jobs n: u32
//!     }
//! }
//! ```
//!
//! Arguments without `--` in then are are positional.
//!
//! ```
//! use std::{path::PathBuf, ffi::OsString};
//!
//! xflags::xflags! {
//!     cmd positional-arguments {
//!         required program: PathBuf
//!         repeated args: OsString
//!     }
//! }
//! ```
//!
//! You can create aliases if desired, which is as simple as adding extra names to the `cmd` definition.
//! In this case, `run` can be called as `run`, `r` and `exec`:
//!
//! ```rust
//! xflags::xflags! {
//!     cmd run r exec {}
//! }
//! ```
//!
//! Nesting **cmd** is allowed. `xflag` automatically generates boilerplate
//! enums for subcommands:
//!
//! ```ignore
//! xflags::xflags! {
//!     src "./examples/subcommands.rs"
//!     cmd app {
//!         repeated -v, --verbose
//!         cmd foo { optional -s, --switch }
//!         cmd bar {}
//!     }
//! }
//!
//! // generated start
//! // The following code is generated by `xflags` macro.
//! // Run `env UPDATE_XFLAGS=1 cargo build` to regenerate.
//! #[derive(Debug)]
//! pub struct App {
//!     pub verbose: u32,
//!     pub subcommand: AppCmd,
//! }
//!
//! #[derive(Debug)]
//! pub enum AppCmd {
//!     Foo(Foo),
//!     Bar(Bar),
//! }
//!
//! #[derive(Debug)]
//! pub struct Foo {
//!     pub switch: bool,
//! }
//!
//! #[derive(Debug)]
//! pub struct Bar {
//! }
//!
//! impl App {
//!     pub fn from_env_or_exit() -> Self {
//!         Self::from_env_or_exit_()
//!     }
//!     pub fn from_env() -> xflags::Result<Self> {
//!         Self::from_env_()
//!     }
//!     pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
//!         Self::from_vec_(args)
//!     }
//! }
//! // generated end
//! ```
//!
//! Switches are always "inherited". Both `app -v foo` and `app foo -v` produce
//! the same result.
//!
//! To make subcommand name optional use the **default** keyword to mark a
//! subcommand to select if no subcommand name is passed. The name of the
//! default subcommand affects only the name of the generated Rust struct, it
//! can't be specified explicitly on the command line.
//!
//! ```
//! xflags::xflags! {
//!     cmd app {
//!         repeated -v, --verbose
//!         default cmd foo { optional -s, --switch }
//!         cmd bar {}
//!     }
//! }
//! ```
//!
//! Commands, arguments, and switches can be documented. Doc comments become a
//! part of generated help:
//!
//! ```
//! mod flags {
//!     use std::path::PathBuf;
//!
//!     xflags::xflags! {
//!         /// Run basic system diagnostics.
//!         cmd healthck {
//!             /// Optional configuration file.
//!             optional config: PathBuf
//!             /// Verbosity level, can be repeated multiple times.
//!             repeated -v, --verbose
//!         }
//!     }
//! }
//!
//! fn main() {
//!     match flags::Healthck::from_env() {
//!         Ok(flags) => {
//!             run_checks(flags.config, flags.verbose);
//!         }
//!         Err(err) => err.exit()
//!     }
//! }
//!
//! # fn run_checks(_config: Option<std::path::PathBuf>, _verbosity: u32) {}
//! ```
//!
//! The **src** keyword controls how the code generation works. If it is absent,
//! `xflags` acts as a typical procedure macro, which generates a bunch of
//! structs and impls.
//!
//! If the **src** keyword is present, it should specify the path to the file
//! with `xflags!` invocation. The path should be relative to the directory with
//! Cargo.toml. The macro then will avoid generating the structs. Instead, if
//! the `UPDATE_XFLAGS` environmental variable is set, the macro will write them
//! directly to the specified file.
//!
//! By convention, `xflag!` macro should be invoked from the `flags` submodule.
//! The `flags::` prefix should be used to refer to command names. Additional
//! validation logic can go to the `flags` module:
//!
//! ```
//! mod flags {
//!     xflags::xflags! {
//!         cmd my-command {
//!             repeated -v, --verbose
//!             optional -q, --quiet
//!         }
//!     }
//!
//!     impl MyCommand {
//!         fn validate(&self) -> xflags::Result<()> {
//!             if self.quiet && self.verbose > 0 {
//!                 return Err(xflags::Error::new(
//!                     "`-q` and `-v` can't be specified at the same time"
//!                 ));
//!             }
//!             Ok(())
//!         }
//!     }
//! }
//! ```
//!
//! The `parse_or_exit!` macro is a syntactic sure for `xflags!`, which
//! immediately parses the argument, exiting the process if needed.
//! `parse_or_exit` only supports single top-level command and doesn't need the
//! `cmd`  keyword.
//!
//! ## Limitations
//!
//! `xflags` follows
//! [Fuchsia](https://fuchsia.dev/fuchsia-src/development/api/cli#command_line_arguments)
//! conventions for command line arguments. GNU conventions such as grouping
//! short-flags (`-xyz`) or gluing short flag and a value `(-fVAL)` are not
//! supported.
//!
//! `xflags` requires the command line interface to be fully static. It's
//! impossible to include additional flags at runtime.
//!
//! Implementation is not fully robust, there might be some residual bugs in
//! edge cases.

use std::fmt;

/// Generates a parser for command line arguments from a DSL.
///
/// See the module-level for detailed syntax specification.
pub use xflags_macros::{parse_or_exit, xflags};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// An error occurred when parssing command line arguments.
///
/// Either the command line was syntactically invalid, or `--help` was
/// explicitly requested.
#[derive(Debug)]
pub struct Error {
    msg: String,
    help: bool,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.msg, f)
    }
}

impl std::error::Error for Error {}

impl Error {
    /// Creates a new `Error` from a given message.
    ///
    /// Use this to report custom validation errors.
    pub fn new(message: impl Into<String>) -> Error {
        Error { msg: message.into(), help: false }
    }

    /// Error that carries `--help` message.
    pub fn is_help(&self) -> bool {
        self.help
    }

    /// Prints the error and exists the process.
    pub fn exit(self) -> ! {
        if self.is_help() {
            println!("{self}");
            std::process::exit(0)
        } else {
            eprintln!("{self}");
            std::process::exit(2)
        }
    }

    /// Appends to the contained message
    pub fn chain(mut self, msg: &str) -> Self {
        self.msg.push_str(msg);
        self
    }
}

/// Private impl details for macros.
#[doc(hidden)]
pub mod rt;
