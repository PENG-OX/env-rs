//! Env Switcher - A lightweight multi-version environment manager
//!
//! Supports Node.js and Java version switching with path-based auto-detection

mod config;
mod matcher;
mod injector;
mod node;
mod java;

pub use config::{Config, PathMapping, VersionConfig};
pub use matcher::PathMatcher;
pub use injector::HookInjector;
pub use node::NodeManager;
pub use java::JavaManager;
