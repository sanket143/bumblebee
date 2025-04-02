//! Bumblebee - A JavaScript code analysis tool
//!
//! This crate provides functionality for analyzing JavaScript code, finding references,
//! and tracking symbol usage across a codebase.

use anyhow::Result;
use bumblebee::cli::{run, Args};
use clap::Parser as ClapParser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    run(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_test() {
        let args = Args {
            project_path: "test-dir".to_string(),
            target_path: "output".to_string(),
        };
        assert!(run(args).is_ok());
    }
}
