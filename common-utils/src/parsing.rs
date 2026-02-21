use crate::logging::Logger;
use crate::InternalIdTooling;
use std::collections::HashMap;
use std::fmt;
use std::ops::Not;
use std::slice::Iter;

#[derive(Clone, PartialEq, Ord, PartialOrd, Eq)]
pub enum ParamOption {
    Value,
    Option,
}
#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ParseError {
    pub fn new(message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
        }
    }
}
#[derive(Clone)]
pub struct ProgramParameter {
    param_type: ParamOption,
    param_value: String,
    param_description: String,
}
pub struct ParameterList {
    list: Vec<ProgramParameter>,
}
impl ProgramParameter {
    pub fn new(param_type: ParamOption, param_value: &str, param_description: &str) -> ProgramParameter {
        ProgramParameter {
            param_type,
            param_value: param_value.to_string(),
            param_description: param_description.to_string(),
        }
    }
}
impl ParameterList {
    pub fn new() -> ParameterList {
        ParameterList {
            list: Vec::new(),
        }
    }
    pub fn push(&mut self, parameter: ProgramParameter) {
        self.list.push(parameter);
    }
    pub fn get_as_map(&self) -> HashMap<String, ParamOption> {
        let mut map: HashMap<String, ParamOption> = HashMap::new();
        for param in &self.list {
            map.insert(param.param_value.clone(), param.param_type.clone());
        }
        map
    }
}
pub struct ArgsMap {
    base_map: HashMap<String, String>,
}
impl ArgsMap {
    pub fn new() -> ArgsMap {
        ArgsMap { base_map: HashMap::new() }
    }
    pub fn contains(&self, search: &str) -> bool {
        self.base_map.clone().contains_key(search)
    }
    pub fn get(&self, search: &str) -> Option<&String> {
        self.base_map.get(search)
    }
    pub fn insert(&mut self, key: String, value: String) {
        self.base_map.insert(key.to_string(), value.to_string());
    }
}
pub struct InternalInlineParsingTool;
impl InternalInlineParsingTool {
    pub fn parse_args(arg_iter: &mut Iter<String>, commands: &ParameterList) -> Result<HashMap<String, String>, ParseError> {
        Logger::trace("Received arguments.. will try matching with commands");
        let mut map: HashMap<String, String> = HashMap::new();
        let mut next_iter_allowed_to_fail: bool = false;
        let mut skip_to_next_iter: bool = false;
        let mut arg_holder: &str;
        let match_commands = commands.get_as_map();
        while arg_iter.as_ref().iter().peekable().peek().is_some() {
            let arg = arg_iter.next().unwrap();
            arg_holder = InternalInlineParsingTool::handle_key_value_param(arg, &mut next_iter_allowed_to_fail, &mut skip_to_next_iter, &match_commands)?;
            if skip_to_next_iter {
                InternalInlineParsingTool::add_to_args(&mut map, arg.to_string(), InternalIdTooling::new_compact_id())?;
                continue;
            }
            if next_iter_allowed_to_fail.not() {
                if arg_iter.as_ref().iter().peekable().peek().is_none() {
                    return Err(ParseError::new(format!("Unable to parse param: \"{}\"! Parameter needs option!", arg).as_str()));
                }
            }
            InternalInlineParsingTool::add_to_args(&mut map, arg_holder.to_string(), arg_iter.next().unwrap().to_string())?;
            next_iter_allowed_to_fail = true;
        }
        Ok(map)
    }
    fn add_to_args(map: &mut HashMap<String, String>, key: String, value: String) -> Result<(), ParseError> {
        if map.contains_key(&key) {
            return Err(ParseError::new(format!("Duplicate key: \"{}\"! Argument is only allowed once!", key).as_str()));
        }
        map.insert(key, value);
        Ok(())
    }
    fn clone(self) {

    }
    fn handle_key_value_param<'a>(
        arg_string: &'a str,
        next_iter_allowed_to_fail: &mut bool,
        skip_to_next_iter: &mut bool,
        match_commands: &HashMap<String, ParamOption>
    ) -> Result<&'a str, ParseError> {
        if !match_commands.contains_key(arg_string.trim()) || match_commands.get(arg_string.trim()).is_none() {
            return Err(ParseError::new(format!("\"{}\" does not seem to be a valid command!", arg_string.trim()).as_str()));
        }
        *skip_to_next_iter = match match_commands.get(arg_string.trim()).unwrap() {
            ParamOption::Value => false,
            ParamOption::Option => true
        };

        if *skip_to_next_iter {
            return Ok(arg_string);
        }

        *next_iter_allowed_to_fail = false;
        Ok(arg_string)
    }
}
pub fn print_usage(args_map: &ParameterList) {
    let mut value_params = Vec::new();
    let mut options = Vec::new();
    for param in &args_map.list {
        match param.param_type {
            ParamOption::Value => value_params.push(param),
            ParamOption::Option => options.push(param)
        }
    }
    println!("\nPROGRAM USAGE:");
    for _number in (1..10) {
        print!("=");
    }
    print!("\n");
    println!("Basic usage: ./acme-sentry-rs [options...] [<parameter> <value>]\n");
    println!("Options:");
    for param in &args_map.list {
        if param.param_type == ParamOption::Option {
            println!("\t{}\t\t\t\t\t{}", param.param_value, param.param_description);
        }
    }
    println!("Parameters:");
    for param in &args_map.list {
        if param.param_type == ParamOption::Value {
            println!("\t{}: <data>\t\t\t{}", param.param_value, param.param_description);
        }
    }
}