//! This crates provides a procedural macro for parsing command line arguments.
//!
//! It is intended for use in development tools, so it emphasizes fast compile
//! times and convenience at the expense of features.
//!
//! Rough decision tree for picking argument parsing library:
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
//! ## Example
//!
//! ```
//! mod flags {
//!     use std::path::PathBuf;
//!
//!     xflags::xflags! {
//!         src "./examples/basic.rs"
//!
//!         cmd my-command
//!             required path: PathBuf
//!         {
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
//!
//!         pub verbose: bool,
//!     }
//!
//!     impl MyCommand {
//!         pub const HELP: &'static str = Self::HELP_;
//!
//!         pub fn from_env() -> xflags::Result<Self> {
//!             Self::from_env_()
//!         }
//!
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
//! To make the macro less opaque, `xflag` can generate `struct` describing the
//! CLI in-place. To disable this behavior, omit the `src` attribute.
//!
//! xflags correctly handles non-utf8 arguments.
//!
//! ## Syntax Reference
//!
//! The **cmd** keyword introduces a command that accepts positional arguments
//! and switches.
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
//!         optional -h, --help
//!         repeated --verbose
//!         required --pass-me
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
//! Positional arguments are specified before the opening curly brace:
//!
//! ```
//! use std::{path::PathBuf, ffi::OsString};
//!
//! xflags::xflags! {
//!     cmd positional-arguments
//!         required program: PathBuf
//!         repeated args: OsString
//!     { }
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
//!     pub const HELP: &'static str = Self::HELP_;
//!
//!     pub fn from_env() -> xflags::Result<Self> {
//!         Self::from_env_()
//!     }
//!
//!     pub fn from_vec(args: Vec<std::ffi::OsString>) -> xflags::Result<Self> {
//!         Self::from_vec_(args)
//!     }
//! }
//! // generated end
//! ```
//!
//! Switches are always "inherited". That is, both `app init --home tmp` and
//! `app --home tmp init` produce the same result given the following
//! definition:
//!
//! ```
//! xflags::xflags! {
//!    cmd app {
//!       optional --home: PathBuf
//!       cmd init { }
//!    }
//! }
//! ```
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
//! Commands, arguments, and switches can documented. Doc comments become a part
//! of generated help:
//!
//! ```
//! mod flags {
//!     use std::path::PathBuf;
//!
//!     xflags::xflags! {
//!         /// Run basic system diagnostics.
//!         cmd healthck
//!             /// Optional configuration file.
//!             optional config: PathBuf
//!         {
//!             /// Verbosity level, can be repeated multiple times.
//!             repeated -v, --verbose
//!             /// Print the help message.
//!             optional -h, --help
//!         }
//!     }
//! }
//!
//! fn main() {
//!     match flags::Healthck::from_env() {
//!         Ok(flags) => {
//!             if flags.help {
//!                 println!("{}", flags::Healthck::HELP);
//!                 return;
//!             }
//!             run_checks(flags.config, flags.verbose);
//!         }
//!         Err(err) => {
//!             eprintln!("{}", err);
//!         }
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

use std::fmt;

/// Generates a parser for command line arguments from a DSL.
///
/// See the module-level for detailed syntax specification.
pub use xflags_macros::xflags;

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// This type represents an error that can occur during command line argument
/// parsing.
#[derive(Debug)]
pub struct Error {
    msg: String,
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
        Error { msg: message.into() }
    }
}

/// Private impl details for macros.
#[doc(hidden)]
pub mod rt;
