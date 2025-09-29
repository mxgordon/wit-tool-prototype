use std::fs::read_to_string;
use std::path::PathBuf;
use clap::{arg, Args};
use clap::{Parser, Subcommand};
use tree_sitter::{Parser as TreeSitterParser, Query, QueryCursor, StreamingIterator, Tree};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(value_name = "FILE")]
    file_path: PathBuf,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
enum Commands {
    AST,
    JSON,
    Query(QueryArgs)
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct QueryArgs {
    query: String,
}

fn handle_query(args: QueryArgs, tree: Tree, file_data: String) {
    let query = Query::new(&tree_sitter_wit::language(), args.query.trim()).unwrap(); //TODO throw clap error

    let mut query_cursor = QueryCursor::new();

    let all_matches = query_cursor.matches(&query, tree.root_node(), file_data.as_bytes());

    all_matches.for_each(|match_| {
        for capture in match_.captures {
            println!("Found {:?} at {:?}", file_data.get(capture.node.byte_range()).unwrap(), capture.node.byte_range());
        }
    });
}

fn main() {
    let cli = Cli::parse();

    let file_data = read_to_string(cli.file_path).unwrap();

    let mut parser = TreeSitterParser::new();

    parser.set_language(&tree_sitter_wit::language()).expect("Set language failed");

    let tree = parser.parse(file_data.as_str(), None).unwrap();

    match cli.command {
        Commands::AST => {todo!()}
        Commands::JSON => {todo!()}
        Commands::Query(query_args) => {handle_query(query_args, tree, file_data)}
    }
}