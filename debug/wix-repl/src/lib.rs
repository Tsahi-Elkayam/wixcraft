//! wix-repl - Interactive REPL for WiX development and testing
//!
//! Provides an interactive environment for exploring and testing WiX concepts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// REPL command type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Evaluate WiX expression
    Eval(String),
    /// Load WiX file
    Load(String),
    /// Show variable or property
    Show(String),
    /// Set variable
    Set(String, String),
    /// List items
    List(ListTarget),
    /// Generate GUID
    Guid,
    /// Show help
    Help,
    /// Clear screen
    Clear,
    /// Exit REPL
    Exit,
    /// Unknown command
    Unknown(String),
}

/// List target types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListTarget {
    Variables,
    Properties,
    Components,
    Features,
    Files,
}

/// REPL context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplContext {
    pub variables: HashMap<String, String>,
    pub properties: HashMap<String, String>,
    pub loaded_files: Vec<String>,
    pub history: Vec<String>,
}

impl ReplContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.variables.insert(name.to_string(), value.to_string());
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    pub fn set_property(&mut self, name: &str, value: &str) {
        self.properties.insert(name.to_string(), value.to_string());
    }

    pub fn get_property(&self, name: &str) -> Option<&String> {
        self.properties.get(name)
    }

    pub fn add_to_history(&mut self, command: &str) {
        self.history.push(command.to_string());
    }

    pub fn clear(&mut self) {
        self.variables.clear();
        self.properties.clear();
        self.loaded_files.clear();
    }
}

/// Command parser
pub struct CommandParser;

impl CommandParser {
    pub fn parse(input: &str) -> Command {
        let input = input.trim();

        if input.is_empty() {
            return Command::Unknown(String::new());
        }

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd = parts[0].to_lowercase();
        let arg = parts.get(1).map(|s| s.trim().to_string());

        match cmd.as_str() {
            "eval" | "e" => Command::Eval(arg.unwrap_or_default()),
            "load" | "l" => Command::Load(arg.unwrap_or_default()),
            "show" | "s" => Command::Show(arg.unwrap_or_default()),
            "set" => {
                if let Some(arg) = arg {
                    let set_parts: Vec<&str> = arg.splitn(2, '=').collect();
                    if set_parts.len() == 2 {
                        Command::Set(set_parts[0].trim().to_string(), set_parts[1].trim().to_string())
                    } else {
                        Command::Unknown(input.to_string())
                    }
                } else {
                    Command::Unknown(input.to_string())
                }
            }
            "list" | "ls" => {
                let target = match arg.as_deref() {
                    Some("vars") | Some("variables") => ListTarget::Variables,
                    Some("props") | Some("properties") => ListTarget::Properties,
                    Some("comps") | Some("components") => ListTarget::Components,
                    Some("features") => ListTarget::Features,
                    Some("files") => ListTarget::Files,
                    _ => ListTarget::Variables,
                };
                Command::List(target)
            }
            "guid" | "g" => Command::Guid,
            "help" | "h" | "?" => Command::Help,
            "clear" | "cls" => Command::Clear,
            "exit" | "quit" | "q" => Command::Exit,
            _ => Command::Unknown(input.to_string()),
        }
    }
}

/// REPL executor
pub struct ReplExecutor;

impl ReplExecutor {
    pub fn execute(command: &Command, context: &mut ReplContext) -> ExecutionResult {
        match command {
            Command::Eval(expr) => {
                // Simple expression evaluation
                let result = Self::eval_expression(expr, context);
                ExecutionResult::output(&result)
            }
            Command::Load(path) => {
                context.loaded_files.push(path.clone());
                ExecutionResult::output(&format!("Loaded: {}", path))
            }
            Command::Show(name) => {
                if let Some(value) = context.get_variable(name) {
                    ExecutionResult::output(&format!("{} = {}", name, value))
                } else if let Some(value) = context.get_property(name) {
                    ExecutionResult::output(&format!("{} = {}", name, value))
                } else {
                    ExecutionResult::error(&format!("Not found: {}", name))
                }
            }
            Command::Set(name, value) => {
                context.set_variable(name, value);
                ExecutionResult::output(&format!("Set {} = {}", name, value))
            }
            Command::List(target) => {
                let items = match target {
                    ListTarget::Variables => Self::format_map(&context.variables),
                    ListTarget::Properties => Self::format_map(&context.properties),
                    _ => "Not implemented".to_string(),
                };
                ExecutionResult::output(&items)
            }
            Command::Guid => {
                let guid = Self::generate_guid();
                ExecutionResult::output(&guid)
            }
            Command::Help => ExecutionResult::output(Self::help_text()),
            Command::Clear => ExecutionResult::clear(),
            Command::Exit => ExecutionResult::exit(),
            Command::Unknown(input) => {
                if input.is_empty() {
                    ExecutionResult::empty()
                } else {
                    ExecutionResult::error(&format!("Unknown command: {}", input))
                }
            }
        }
    }

    fn eval_expression(expr: &str, context: &ReplContext) -> String {
        // Simple variable substitution
        let mut result = expr.to_string();
        for (name, value) in &context.variables {
            result = result.replace(&format!("${{{}}}", name), value);
        }
        for (name, value) in &context.properties {
            result = result.replace(&format!("[{}]", name), value);
        }
        result
    }

    fn format_map(map: &HashMap<String, String>) -> String {
        if map.is_empty() {
            return "(empty)".to_string();
        }
        map.iter()
            .map(|(k, v)| format!("  {} = {}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn generate_guid() -> String {
        // Simple GUID generation (would use uuid crate in production)
        "{00000000-0000-0000-0000-000000000000}".to_string()
    }

    fn help_text() -> &'static str {
        r#"WiX REPL Commands:
  eval <expr>     - Evaluate expression with variable substitution
  load <file>     - Load WiX source file
  show <name>     - Show variable or property value
  set <name>=<val>- Set variable
  list <target>   - List items (vars, props, comps, features, files)
  guid            - Generate a new GUID
  help            - Show this help
  clear           - Clear screen
  exit            - Exit REPL"#
    }
}

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub output: Option<String>,
    pub error: Option<String>,
    pub should_exit: bool,
    pub should_clear: bool,
}

impl ExecutionResult {
    pub fn output(msg: &str) -> Self {
        Self {
            output: Some(msg.to_string()),
            error: None,
            should_exit: false,
            should_clear: false,
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            output: None,
            error: Some(msg.to_string()),
            should_exit: false,
            should_clear: false,
        }
    }

    pub fn exit() -> Self {
        Self {
            output: Some("Goodbye!".to_string()),
            error: None,
            should_exit: true,
            should_clear: false,
        }
    }

    pub fn clear() -> Self {
        Self {
            output: None,
            error: None,
            should_exit: false,
            should_clear: true,
        }
    }

    pub fn empty() -> Self {
        Self {
            output: None,
            error: None,
            should_exit: false,
            should_clear: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = ReplContext::new();
        assert!(ctx.variables.is_empty());
    }

    #[test]
    fn test_context_set_variable() {
        let mut ctx = ReplContext::new();
        ctx.set_variable("test", "value");
        assert_eq!(ctx.get_variable("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_context_set_property() {
        let mut ctx = ReplContext::new();
        ctx.set_property("PROP", "val");
        assert_eq!(ctx.get_property("PROP"), Some(&"val".to_string()));
    }

    #[test]
    fn test_context_history() {
        let mut ctx = ReplContext::new();
        ctx.add_to_history("test command");
        assert_eq!(ctx.history.len(), 1);
    }

    #[test]
    fn test_parse_eval() {
        let cmd = CommandParser::parse("eval test");
        assert_eq!(cmd, Command::Eval("test".to_string()));
    }

    #[test]
    fn test_parse_load() {
        let cmd = CommandParser::parse("load file.wxs");
        assert_eq!(cmd, Command::Load("file.wxs".to_string()));
    }

    #[test]
    fn test_parse_show() {
        let cmd = CommandParser::parse("show MyVar");
        assert_eq!(cmd, Command::Show("MyVar".to_string()));
    }

    #[test]
    fn test_parse_set() {
        let cmd = CommandParser::parse("set name=value");
        assert_eq!(cmd, Command::Set("name".to_string(), "value".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let cmd = CommandParser::parse("list vars");
        assert_eq!(cmd, Command::List(ListTarget::Variables));
    }

    #[test]
    fn test_parse_guid() {
        let cmd = CommandParser::parse("guid");
        assert_eq!(cmd, Command::Guid);
    }

    #[test]
    fn test_parse_help() {
        let cmd = CommandParser::parse("help");
        assert_eq!(cmd, Command::Help);
    }

    #[test]
    fn test_parse_exit() {
        let cmd = CommandParser::parse("exit");
        assert_eq!(cmd, Command::Exit);
    }

    #[test]
    fn test_parse_unknown() {
        let cmd = CommandParser::parse("unknown");
        assert!(matches!(cmd, Command::Unknown(_)));
    }

    #[test]
    fn test_execute_set() {
        let mut ctx = ReplContext::new();
        let result = ReplExecutor::execute(&Command::Set("x".to_string(), "1".to_string()), &mut ctx);
        assert!(result.output.is_some());
        assert_eq!(ctx.get_variable("x"), Some(&"1".to_string()));
    }

    #[test]
    fn test_execute_show() {
        let mut ctx = ReplContext::new();
        ctx.set_variable("x", "1");
        let result = ReplExecutor::execute(&Command::Show("x".to_string()), &mut ctx);
        assert!(result.output.unwrap().contains("1"));
    }

    #[test]
    fn test_execute_guid() {
        let mut ctx = ReplContext::new();
        let result = ReplExecutor::execute(&Command::Guid, &mut ctx);
        assert!(result.output.unwrap().contains("{"));
    }

    #[test]
    fn test_execute_exit() {
        let mut ctx = ReplContext::new();
        let result = ReplExecutor::execute(&Command::Exit, &mut ctx);
        assert!(result.should_exit);
    }

    #[test]
    fn test_execute_clear() {
        let mut ctx = ReplContext::new();
        let result = ReplExecutor::execute(&Command::Clear, &mut ctx);
        assert!(result.should_clear);
    }

    #[test]
    fn test_execution_result_output() {
        let r = ExecutionResult::output("test");
        assert_eq!(r.output, Some("test".to_string()));
        assert!(!r.should_exit);
    }

    #[test]
    fn test_execution_result_error() {
        let r = ExecutionResult::error("err");
        assert_eq!(r.error, Some("err".to_string()));
    }
}
