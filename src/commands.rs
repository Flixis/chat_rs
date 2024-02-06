pub trait Command {
    // Trait method to execute the command, to be implemented by each command.
    fn execute(&self);
}

// The CommandInfo struct holds information about the command.
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub privilege_level: Option<usize>,
}

impl CommandInfo {
    // Constructor for CommandInfo
    pub fn new(name: &str, description: &str, privilege_level: Option<usize>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            privilege_level,
        }
    }
}

// Implement the Command trait for CommandInfo to print a basic message.
// This is a default behavior that can be overridden by specific commands.
impl Command for CommandInfo {
    fn execute(&self) {
        println!("Default command execution: {}", self.name);
        // Default execution behavior can be defined here
    }
}

// Now, you can create specific commands by creating structs for them and implementing the Command trait.
pub struct GreetCommand {
    pub info: CommandInfo,
}

impl GreetCommand {
    pub fn new() -> Self {
        Self {
            info: CommandInfo::new("Greet", "Sends a greeting message", Some(1)),
        }
    }
}

impl Command for GreetCommand {
    fn execute(&self) {
        println!("Hello! This is the '{}' command.", self.info.name);
        // Implement the greeting command functionality here
    }
}