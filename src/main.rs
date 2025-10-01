use std::fs::read_to_string;
use std::path::PathBuf;
use clap::{arg, Args};
use clap::{Parser, Subcommand};
use tree_sitter::{Parser as TreeSitterParser, Query, QueryCursor, StreamingIterator, Tree};
use std::fs::File;
use std::io::Write;

mod tree_to_json;

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
    Tokens,
    AST,
    JSON,
    Query(QueryArgs)
}

#[derive(Args, Debug, Clone, PartialEq, Eq)]
struct QueryArgs {
    query: String,
}

fn handle_query(args: QueryArgs, tree: Tree, file_data: String) {
    // Create a treesitter query using the query syntax from treesitter
    let query = Query::new(&tree_sitter_wit::language(), args.query.trim()).unwrap(); //TODO throw clap error if query fails to parse

    let mut query_cursor = QueryCursor::new();

    // Run the query on the tree
    let all_matches = query_cursor.matches(&query, tree.root_node(), file_data.as_bytes());

    // Print the section of the WIT file it matched and the location
    all_matches.for_each(|match_| {
        for capture in match_.captures {
            println!("Found {:?} at {:?}", file_data.get(capture.node.byte_range()).unwrap(), capture.node.byte_range());
        }
    });
}

fn handle_tokens(tree: Tree, file_data: String) {
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

fn handle_json(tree: Tree, file_data: String) {
    let mut cursor = tree.walk();
    let mut file = File::create("test.json").expect("Failed to create file");
    let mut json_string = String::new();
    let mut intfs = vec![];
    let mut wrlds = vec![];

    'all: loop {
        let node = cursor.node();
        // let start_byte = node.start_byte();
        // let end_byte = node.end_byte();
        // let node_text = &(file_data.as_str())[start_byte..end_byte];
        //println!("{:indent$}Node Type: {:?}, {:?}", "", node.kind(), node_text, indent = depth * 2);

        if node.kind() == "interface_item" {
            let intf = tree_to_json::parse_interface(&(file_data.as_str()), node);
            intfs.push(intf);
            //let intf_json = serde_json::to_string_pretty(&intf).unwrap();

            //json_string += &(intf_json + "\n");
        }

        // Try to go to the first child
        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() {
                break 'all;
            }

            if cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let wit_file = tree_to_json::WitFile {
        interfaces: intfs,
        worlds: wrlds
    };
    let json_string = serde_json::to_string_pretty(&wit_file).unwrap();
    file.write_all(json_string.as_bytes()).expect("Failed to write to file");
}

fn main() {
    // Parse CLI input from user
    let cli = Cli::parse();

    let file_data = read_to_string(cli.file_path).unwrap();

    // Create treesitter parser and parse WIT file
    let mut parser = TreeSitterParser::new();
    parser.set_language(&tree_sitter_wit::language()).expect("Set language failed");
    let tree = parser.parse(file_data.as_str(), None).unwrap();

    // Based on user input, run appropriate function
    match cli.command {
        Commands::Tokens => handle_tokens(tree, file_data),
        Commands::AST => println!("{:?}", tree),
        Commands::JSON => handle_json(tree, file_data),
        Commands::Query(query_args) => {handle_query(query_args, tree, file_data)}
    }
}