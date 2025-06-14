use crate::InternalIdTooling;
use std::collections::HashMap;
use std::fmt;
use std::ops::Not;
use std::slice::Iter;

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
pub struct InternalInlineParsingTool;
impl InternalInlineParsingTool {
    pub fn parse_args(arg_iter: &mut Iter<String>, match_commands: &HashMap<String, ParamOption>) -> Result<HashMap<String, String>, ParseError> {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut next_iter_allowed_to_fail: bool = false;
        let mut skip_to_next_iter: bool = false;
        let mut arg_holder: &str;
        while arg_iter.as_ref().iter().peekable().peek().is_some() {
            let arg = arg_iter.next().unwrap();
            arg_holder = InternalInlineParsingTool::handle_key_value_param(arg, &mut next_iter_allowed_to_fail, &mut skip_to_next_iter, &match_commands)?;
            if skip_to_next_iter {
                InternalInlineParsingTool::add_to_args(&mut map, arg.to_string(), InternalIdTooling::new_compact_id());
                continue;
            }
            if next_iter_allowed_to_fail.not() {
                if arg_iter.as_ref().iter().peekable().peek().is_none() {
                    return Err(ParseError::new("Unable to parse param: \"{}\"! Parameter needs option!"));
                }
            }
            InternalInlineParsingTool::add_to_args(&mut map, arg_holder.to_string(), arg_iter.next().unwrap().to_string());
            next_iter_allowed_to_fail = true;
        }
        Ok(map)
    }
    fn add_to_args(map: &mut HashMap<String, String>, key: String, value: String) {
        if map.contains_key(&key) {
            panic!("Duplicate key: \"{}\"! Argument is only allowed once!", key);
        }
        map.insert(key, value);
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