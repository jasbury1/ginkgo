use crate::model::Model;

pub enum Command {
    InsertString {
        location: (usize, usize),
        contents: String,
    },
    DeleteString {
        start: (usize, usize),
        end: (usize, usize),
    },
    InsertChar {
        location: (usize, usize),
        c: char,
    },
    DeleteChar {
        location: (usize, usize),
    },
}

impl Command {
    pub fn execute(&self, model: &mut Model) -> Command{
        match self {
            Command::InsertString { location, contents } => todo!(),
            Command::DeleteString { start, end } => todo!(),
            Command::InsertChar { location, c } => {
                model.cx = location.0;
                model.cy = location.1;
                model.insert_char(*c);
                Command::DeleteChar {
                    location: (model.cx, model.cy),
                }
            }
            Command::DeleteChar { location } => {
                model.cx = location.0;
                model.cy = location.1;
                let chr: char = model.get_char();
                model.delete_char();
                Command::InsertChar {
                    location: (model.cx, model.cy),
                    c: chr,
                }
            }
        }
    }
}

pub struct CommandState {
    undo_commands: Vec<Command>,
    undo_steps: Vec<usize>,
    redo_commands: Vec<Command>,
    redo_steps: Vec<usize>,
}

impl CommandState {
    pub fn new() -> CommandState {
        CommandState {
            // Vec of the command steps for each undo
            undo_commands: Vec::new(),
            // Number of simultaneous steps to perform a single undo
            undo_steps: Vec::new(),
            // Vec of the command steps for each redo
            redo_commands: Vec::new(),
            // Number of simultaneous steps to perform a single redo
            redo_steps: Vec::new(),
        }
    }

    pub fn execute_command(&mut self, cmd: Command, model: &mut Model) {
        let undo_cmd = cmd.execute(model);
        self.redo_commands.clear();
        self.redo_steps.clear();
        self.undo_commands.push(undo_cmd);
        self.undo_steps.push(1);
    }

    pub fn execute_command_group(&mut self, cmds: &mut Vec<Command>, model: &mut Model) {
        let len = cmds.len();
        for cmd in cmds {
            let undo_cmd = cmd.execute(model);
            self.undo_commands.push(undo_cmd);

        }
        self.redo_commands.clear();
        self.redo_steps.clear();
        self.undo_steps.push(len);
    }

    pub fn execute_undo(&mut self, model: &mut Model) {
        // Function becomes noop if undo_steps is empty
        if let Some(len) = self.undo_steps.pop() {
            // Execute one or more undo moves that should happen at once
            for _ in 0..len {
                let cmd = self.undo_commands.pop().unwrap();
                let redo_cmd = cmd.execute(model);
                self.redo_commands.push(redo_cmd);
            }
            self.redo_steps.push(len);
            //TODO: model.dirty -= 1;
        }
    }

    pub fn execute_redo(&mut self, model: &mut Model) {
        //TODO: model.dirty += 1;
        if let Some(len) = self.redo_steps.pop() {
            for _ in 0..len {
                let cmd = self.redo_commands.pop().unwrap();
                let undo_cmd = cmd.execute(model);
                self.undo_commands.push(undo_cmd);
            }
            self.undo_steps.push(len);
        }
    }
}
