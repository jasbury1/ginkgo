use crate::model::Model;

pub trait Command {
    fn execute(&self, model: &mut Model);
}

pub struct InsertString {
    pub location: (usize, usize),
    pub contents: String
}

impl Command for InsertString {
    fn execute(&self, model: &mut Model) {
        model.cx = self.location.0;
        model.cy = self.location.1;
        model.insert_string(&self.contents);
    } 
}

pub struct InsertChar {
    location: (usize, usize),
    c: char
}

pub struct DeleteString {
    location: (usize, usize),
    contents: String
}

pub struct DeleteChar {
    location: (usize, usize),
    c: char
}

pub struct CommandState {
    undo_commands: Vec<Box<dyn Command>>,
    redo_commands: Vec<Box<dyn Command>>,
}

impl CommandState {
    pub fn new() -> CommandState {
        CommandState {
            undo_commands: Vec::new(),
            redo_commands: Vec::new(),
        }
    }

    /// Provide the CommandState with the next Command to execute if the user hits `undo`
    ///
    /// # Arguments
    ///
    /// * `name` - A string slice that holds the name of the person
    ///
    pub fn push_undo(&mut self, cmd: Box<dyn Command>) {
        self.undo_commands.push(cmd);
    }

    pub fn execute_undo(&mut self, model: &mut Model) {
        if let Some(cmd) = self.undo_commands.pop() {
            cmd.execute(model);
        }
    }

    pub fn execute_redo(&mut self, model: &Model) {

    }

}

