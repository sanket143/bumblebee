use crate::core::Bumblebee;
use crate::query::Query;

use anyhow::Result;
use clap::Parser;
use oxc_allocator::Allocator;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub project_path: String,

    #[arg(long, default_value = "../output")]
    pub target_path: String,
}

pub fn run(args: Args) -> Result<()> {
    let mut allocator = Allocator::default();
    let home = std::env::current_dir()?;
    let root_path = home.join(args.project_path);
    let target_dir = Path::new(&args.target_path);
    let mut bumblebee = Bumblebee::new(&root_path, target_dir, &mut allocator);

    eval_dir(&mut bumblebee)
}

fn eval_dir<'a>(bumblebee: &'a mut Bumblebee<'a>) -> Result<()> {
    let queries = [Query::new_with_symbol(
        "call".into(),
        PathBuf::from("./factory.js"),
    )];

    for query in queries {
        println!("{:?}", query);
        bumblebee.evaluate_query(query);
    }

    bumblebee.update_services()?;
    bumblebee.find_references_recursively();
    bumblebee.dump_reference_files();

    Ok(())
}
