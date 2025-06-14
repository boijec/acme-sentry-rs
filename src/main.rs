use std::collections::HashMap;
use std::env;
use common_utils::parsing::{InternalInlineParsingTool, ParamOption};
use common_utils::parsing::ParamOption::{Option, Value};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut argument_iterator = args.iter();
    // skip program name!
    argument_iterator.next().unwrap();
    let mut allowed_args: HashMap<String, ParamOption> = HashMap::new();
    allowed_args.insert("--acme-base-url".to_string(), Value);
    allowed_args.insert("--verbose".to_string(), Value);
    allowed_args.insert("-f".to_string(), Option);
    let map = InternalInlineParsingTool::parse_args(&mut argument_iterator, &allowed_args);
    if map.is_err() {
        eprintln!("{}", map.unwrap_err());
        println!("show usage bumper - then die!");
        return;
    }
    // handle the consequences of the map
    // initiate call to acme ca server for directory fetch
    // save directory to datastore
    // create client / user - keys
    println!("argument map: {:#?}", map);
}
