use common::{cmd::Cmd, logger, spinner::Spinner};
use config::EcosystemConfig;
use xshell::{cmd, Shell};

use crate::messages::{
    MSG_CARGO_NEXTEST_MISSING_ERR, MSG_RUNNING_UNIT_TESTS_SPINNER, MSG_UNIT_TESTS_RUN_SUCCESS,
    MSG_USING_CARGO_NEXTEST,
};

use super::args::rust::RustArgs;

pub fn run(shell: &Shell, args: RustArgs) -> anyhow::Result<()> {
    let ecosystem = EcosystemConfig::from_file(shell)?;
    let _dir_guard = shell.push_dir(&ecosystem.link_to_code);

    let cmd = if nextest_is_installed(shell)? {
        logger::info(MSG_USING_CARGO_NEXTEST);
        cmd!(shell, "cargo nextest run --release")
    } else {
        logger::error(MSG_CARGO_NEXTEST_MISSING_ERR);
        cmd!(shell, "cargo test --release")
    };

    let spinner = Spinner::new(MSG_RUNNING_UNIT_TESTS_SPINNER);
    if let Some(options) = args.options {
        Cmd::new(cmd.args(options.split_whitespace())).run()?;
    } else {
        Cmd::new(cmd).run()?;
    }
    spinner.finish();

    logger::outro(MSG_UNIT_TESTS_RUN_SUCCESS);
    Ok(())
}

fn nextest_is_installed(shell: &Shell) -> anyhow::Result<bool> {
    let out = String::from_utf8(
        Cmd::new(cmd!(shell, "cargo install --list"))
            .run_with_output()?
            .stdout,
    )?;
    Ok(out.contains("cargo-nextest"))
}
