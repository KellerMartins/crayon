use clap;

use errors::*;
use cargo;

use workflow::Workflow;

pub fn execute(workflow: &mut Workflow, matches: &clap::ArgMatches) -> Result<()> {
    workflow.build()?;

    let mut args = vec!["build", "--color=always"];

    if matches.is_present("release") {
        args.push("--release");
    }

    cargo::call(&args)
}