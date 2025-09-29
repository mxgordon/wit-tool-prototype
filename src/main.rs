use std::fmt::Display;
use std::fs::read_to_string;
use std::path::PathBuf;
use clap::{arg, ValueEnum};
use clap::{Parser, Subcommand};
use tree_sitter::Parser as TreeSitterParser;
use serde_json;
use std::fs::File;
use std::io::Write;
mod tree_to_json;

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
        Commands::Query => {
            println!("Not implemented yet");
        }
    }
}