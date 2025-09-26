use std::fmt::Display;
use std::fs::read_to_string;
use std::path::PathBuf;
use clap::{arg, Args, ValueEnum};
use clap::{Parser, Subcommand};
use tree_sitter::Parser as TreeSitterParser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(value_name = "FILE")]
    file_path: PathBuf,

    #[arg(default_value_t = Commands::AST)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq, ValueEnum)]
enum Commands {
    AST,
    JSON,
    Query
}

// #[derive(Args, Debug, Clone, PartialEq, Eq)]
// struct QueryArgs {
//     query: String,
// }

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Commands::AST => "ast",
            Commands::JSON => "json",
            Commands::Query => "query"
        })
    }
}

fn main() {
    let cli = Cli::parse();

    let file_data = read_to_string(cli.file_path).unwrap();

    let mut parser = TreeSitterParser::new();

    parser.set_language(&tree_sitter_wit::language()).expect("Set language failed");

    let tree = parser.parse(file_data.as_str(), None).unwrap();

    print!("{:?}", tree);
}