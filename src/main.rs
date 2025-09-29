use std::fmt::Display;
use std::fs::read_to_string;
use std::path::PathBuf;
use clap::{arg, ValueEnum};
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
    Tokens,
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
            Commands::Tokens => "tokens",
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

    match cli.command {
        Commands::Tokens => {
            // get root node of parsed tree
            let root_node = tree.root_node();

            // create cursor to traverse tree
            let mut cursor = root_node.walk();

            //iterate over all named children of root node (only top-level)
            for node in root_node.named_children(&mut cursor) {
                // print node kind and text
                println!(
                    "Token: {}, Text: {}",
                    node.kind(),
                    &file_data[node.byte_range()]
                );
            }
        }
        Commands::AST => {
            println!("{:?}", tree);
        }
        Commands::JSON => {
            println!("Not implemented yet");
        }
        Commands::Query => {
            println!("Not implemented yet");
        }
    }
}