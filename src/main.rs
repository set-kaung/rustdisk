use clap::Parser;
mod tree;
use std::path::PathBuf;
mod hrsize;
use crate::tree::{Node, Tree, print_entries};
mod error;
use chrono::Local;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(help = "The target directory path", default_value = "./")]
    path: std::path::PathBuf,

    #[arg(short, long, default_value_t = 5, help = "How much depth to show.")]
    level: u8,

    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Shorten the output, control with width option"
    )]
    shorten: bool,

    #[arg(
        short,
        long,
        default_value_t = 20,
        help = "Length to print out the file/dir name."
    )]
    width: u8,

    #[arg(short, long, default_value_t = false, help = "Show directories only")]
    dir_only: bool,
}

fn main() {
    let args = Args::parse();
    let target_path = PathBuf::from(&args.path);
    if !target_path.is_dir() {
        eprintln!("error: {} is not a direcotry.", args.path.display());
        std::process::exit(1);
    }
    let mut tree = Tree::new(Some(Node::new(None, target_path.clone(), 0, true)));

    match tree.build() {
        Ok(()) => {
            let now = Local::now();
            let time_str = format!("{}", now);
            let dashes = "-".repeat(time_str.len() + 1);
            println!("{}\n{}", time_str, dashes);
            print_entries(
                &mut tree.nodes(),
                args.level,
                args.shorten,
                args.width,
                args.dir_only,
            );
        }
        Err(e) => match e {
            error::AppError::NotFound => {
                eprintln!("path not found: {}", target_path.display())
            }
            error::AppError::AccessDenied => eprintln!("access denied: {}", target_path.display()),
            error::AppError::Fatal(s) => eprintln!("unrecoverable error: {}", s),
        },
    }
}
