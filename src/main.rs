use clap::{Args, arg};
use clap::{Parser, Subcommand};
use serde::Serialize;
use std::fmt::Display;
use std::fs::File;
use std::fs::read_to_string;
use std::path::PathBuf;
use tree_sitter::{
    Node, Parser as TreeSitterParser, Point, Query, QueryCursor, StreamingIterator, Tree,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone, PartialEq, Eq)]
enum Commands {
    Tokens(FilePathArgs),
    Ast(FilePathArgs),
    Json(JSONArgs),
    Query(QueryArgs),
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct QueryArgs {
    #[arg(value_name = "FILE")]
    file_path: PathBuf,
    query: String,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct FilePathArgs {
    #[arg(value_name = "FILE")]
    file_path: PathBuf,
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct JSONArgs {
    #[arg(value_name = "FILE")]
    file_path: PathBuf,
    json_file_name: String,
}

impl Display for Commands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Commands::Tokens(..) => "tokens",
                Commands::Ast(..) => "ast",
                Commands::Json(..) => "json",
                Commands::Query(..) => "query",
            }
        )
    }
}

fn handle_query(query_args: QueryArgs) {
    let file_data = read_to_string(query_args.file_path).unwrap();

    let tree = make_tree(&file_data);
    // Create a treesitter query using the query syntax from treesitter
    let query = Query::new(&tree_sitter_wit::language(), query_args.query.trim()).unwrap(); //TODO throw clap error if query fails to parse

    let mut query_cursor = QueryCursor::new();

    // Run the query on the tree
    let all_matches = query_cursor.matches(&query, tree.root_node(), file_data.as_bytes());

    // Print the section of the WIT file it matched and the location
    all_matches.for_each(|match_| {
        for capture in match_.captures {
            println!(
                "Found {:?} at {:?}",
                file_data.get(capture.node.byte_range()).unwrap(),
                capture.node.byte_range()
            );
        }
    });
}

fn handle_tokens(file_args: FilePathArgs) {
    let file_data = read_to_string(file_args.file_path).unwrap();

    let tree = make_tree(&file_data);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub row: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "({},{})", self.row, self.column)
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self {
            row: point.row + 1,
            column: point.column + 1,
        }
    }
}

#[derive(Serialize)]
pub struct SyntaxNode {
    pub kind: String,
    pub text: String,
    pub children: Vec<SyntaxNode>,
}

impl SyntaxNode {
    fn from_node(node: Node, file_data: String) -> Self {
        let mut walker = node.walk();
        // convert all the child node to SyntaxNodes
        let children: Vec<SyntaxNode> = node
            .children(&mut walker)
            .map(|n| SyntaxNode::from_node(n, file_data.clone()))
            .collect();

        // Filter out the children that are just syntax tokens
        let important_children = children
            .into_iter()
            .filter(|child| {
                !["{", "}", ":", "//", ";", "<", ">", "->", "(", ")"].contains(&&*child.kind)
            })
            .collect();

        Self {
            kind: node.kind().into(),
            text: file_data.get(node.byte_range()).unwrap().to_string(),
            children: important_children,
        }
    }
}

fn handle_json(json_args: JSONArgs) {
    let file_data = read_to_string(json_args.file_path).unwrap();

    let tree = make_tree(&file_data);

    let mut file = File::create(json_args.json_file_name).expect("Failed to create file");

    let root: SyntaxNode = SyntaxNode::from_node(tree.root_node(), file_data.clone());

    // Pretty print the JSON file
    serde_json::to_writer_pretty(&mut file, &root).unwrap();
}


fn make_tree(file_data: &str) -> Tree {
    // Create treesitter parser and parse WIT file
    let mut parser = TreeSitterParser::new();
    parser
        .set_language(&tree_sitter_wit::language())
        .expect("Set language failed");

    parser.parse(file_data, None).unwrap()
}

fn main() {
    // Parse CLI input from user
    let cli = Cli::parse();

    // Based on user input, run the appropriate function
    match cli.command {
        Commands::Tokens(file_args) => handle_tokens(file_args),
        Commands::Ast(file_args) => println!("{:?}", make_tree(&read_to_string(file_args.file_path).unwrap())),
        Commands::Json(json_args) => handle_json(json_args),
        Commands::Query(query_args) => handle_query(query_args),
    }
}
