use common_utils::logging::{Logger, LoggingLevel};
use common_utils::parsing::ParamOption::{Option, Value};
use common_utils::parsing::{InternalInlineParsingTool, ParamOption};
use common_utils::InternalIdTooling;
use persistence::database::DatabaseConnection;
use std::collections::HashMap;
use std::fmt::Error;
use std::{env, thread};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut argument_iterator = args.iter();
    let _logger_instance_handler = Logger::new();
    Logger::initialize_with(LoggingLevel::INFO);
    Logger::insert_mdc("id", InternalIdTooling::new_compact_id());
    // skip program name!
    argument_iterator.next().unwrap();
    let mut allowed_args: HashMap<String, ParamOption> = HashMap::new();
    allowed_args.insert("--acme-base-url".to_string(), Value);
    allowed_args.insert("--verbose".to_string(), Value);
    allowed_args.insert("-f".to_string(), Option);
    let map = InternalInlineParsingTool::parse_args(&mut argument_iterator, &allowed_args);
    let thread_join_handle = thread::Builder::new().name("db_initializer".to_string()).spawn(move || {
        let _logger = Logger::new();
        Logger::initialize_with(LoggingLevel::TRACE);
        Logger::insert_mdc("job_id", InternalIdTooling::new_compact_id());
        let db = DatabaseConnection::get_connection().unwrap();
        db.internal_structure_check().unwrap();
        Ok(())
    }).unwrap();
    let _res: Result<_, Error> = thread_join_handle.join().unwrap();
    if map.is_err() {
        Logger::error(&format!("{}", map.unwrap_err()));
        println!("show usage bumper - then die!");
        return;
    }
    Logger::info(&format!("Argument result: {:#?}", map));
}
