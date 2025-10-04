use serde::Serialize;
use tree_sitter::Node;

#[derive(Serialize, Debug)]
pub struct Parameter {
    name: String,
    type_: String,
}

#[derive(Serialize, Debug)]
pub struct Function {
    name: String,
    parameters: Vec<Parameter>,
    returns: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct Interface {
    name: String,
    functions: Vec<Function>,
}

#[derive(Serialize)]
#[serde(untagged)]
#[expect(dead_code)]
enum WorldExport {
    Interface {
        interface: String,
    },
    Function {
        function: String,
        parameters: Vec<Parameter>,
        returns: Option<String>,
    },
}

#[derive(Serialize)]
pub struct World {
    name: String,
    exports: Vec<WorldExport>,
    imports: Vec<String>,
}

#[derive(Serialize)]
pub struct WitFile {
    pub interfaces: Vec<Interface>,
    pub worlds: Vec<World>,
}

fn get_descendants<'a>(node: Node<'a>, kind: &'a str) -> Vec<Node<'a>> {
    let mut cursor = node.walk();
    let mut depth = 0;
    let mut output = vec![];

    'all: loop {
        if cursor.node().kind() == kind {
            output.push(cursor.node());
        }

        // Try to go to the first child
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        loop {
            if !cursor.goto_parent() && depth <= 0 {
                break 'all;
            }
            depth -= 1;

            if cursor.goto_next_sibling() {
                break;
            }
        }
    }

    output
}

fn get_text<'a>(doc: &'a str, node: Node<'a>) -> &'a str {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();

    &doc[start_byte..end_byte]
}

pub fn parse_parameter(doc: &str, param_node: Node) -> Vec<Parameter> {
    let param_names = get_descendants(param_node, "identifier");
    let param_types = get_descendants(param_node, "ty");
    let mut output = vec![];

    for (index, value) in param_names.iter().enumerate() {
        let name_text = get_text(doc, *value);
        let type_text = get_text(doc, param_types[index]);

        //println!("{:?}, {:?}", name_text, type_text);
        output.push(Parameter {
            name: name_text.to_string(),
            type_: type_text.to_string(),
        });
    }

    output
}

pub fn parse_function(doc: &str, func_node: Node) -> Function {
    let func_name = get_text(doc, get_descendants(func_node, "identifier")[0]).to_string();
    let param_list = get_descendants(func_node, "param_list")[0];
    let params = parse_parameter(doc, param_list);

    Function {
        name: func_name,
        parameters: params,
        returns: None,
    }
}

pub fn parse_interface(doc: &str, func_node: Node) -> Interface {
    let interface_name = get_text(doc, get_descendants(func_node, "identifier")[0]).to_string();
    let func_list = get_descendants(func_node, "func_item");
    let mut funcs = vec![];

    for func_node in func_list {
        funcs.push(parse_function(doc, func_node));
    }

    Interface {
        name: interface_name,
        functions: funcs,
    }
}
