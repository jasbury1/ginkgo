use crate::model::Model;

pub enum Command {
    InsertNewline {
        location: (usize, usize)
    },
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
            Command::InsertNewline {location} => {
                model.cx = location.0;
                model.cy = location.1;
                model.insert_newline();
                Command::DeleteChar {
                    location: (model.cx, model.cy)
                }
            },
            Command::InsertString { location, contents } => {
                model.cx = location.0;
                model.cy = location.1;
                model.insert_string(contents);
                Command::DeleteString {
                    start: *location,
                    end: (model.cx, model.cy)
                }
            },
            Command::DeleteString { start, end } => {
                model.anchor_start = *start;
                model.anchor_end = *end;
                let selection: String = model.get_selection();
                model.delete_selection();
                Command::InsertString {
                    location: *start,
                    contents: selection
                }
            }
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
                if chr == '\n' {
                    Command::InsertNewline {
                        location: (model.cx, model.cy)
                    }
                } else {
                    Command::InsertChar {
                        location: (model.cx, model.cy),
                        c: chr,
                    }
                }
            }
        }
    }
}


pub struct CommandState {
    // Vec of the command steps for each undo
    undo_commands: Vec<Command>,
    // Number of simultaneous steps to perform a single undo
    undo_steps: Vec<usize>,
    // Vec of the command steps for each redo
    redo_commands: Vec<Command>,
    // Number of simultaneous steps to perform a single redo
    redo_steps: Vec<usize>,
    // Number of unsaved changes. Can be negative for unsaved undos
    pub change_count: i32
}

impl CommandState {
    pub fn new() -> CommandState {
        CommandState {
            undo_commands: Vec::new(),            
            undo_steps: Vec::new(),           
            redo_commands: Vec::new(),       
            redo_steps: Vec::new(),
            change_count: 0
            
        }
    }

    pub fn execute_command(&mut self, cmd: Command, model: &mut Model) {
        // After a change, unsaved undos count positively
        if self.change_count < 0 {
            self.change_count *= -1;
        }

        let undo_cmd = cmd.execute(model);
        self.redo_commands.clear();
        self.redo_steps.clear();

        // Attempt to merge this command with existing commands, and return early if we can
        if let Command::InsertChar { location, c } = &cmd {
            if self.try_merge_char_cmd(*c, *location, model) {
                 return;
            }
        }

        self.undo_commands.push(undo_cmd);
        self.undo_steps.push(1);

        self.change_count += 1;
    }

    pub fn execute_command_group(&mut self, cmds: &mut Vec<Command>, model: &mut Model) {
        // After a change, unsaved undos count positively
        if self.change_count < 0 {
            self.change_count *= -1;
        }
    
        let len = cmds.len();
        for cmd in cmds {
            let undo_cmd = cmd.execute(model);
            self.undo_commands.push(undo_cmd);

        }
        self.redo_commands.clear();
        self.redo_steps.clear();
        self.undo_steps.push(len);

        self.change_count += 1;
    }

    
    fn try_merge_char_cmd(&mut self, new_c: char, new_location: (usize, usize), model: &mut Model) -> bool {
        if !new_c.is_alphabetic() {
            return false;
        }

        // We can merge strings with extra consecutive characters into a longer string
        if let Some(Command::DeleteString { start: _, end } ) = self.undo_commands.last_mut() {
            if end.0 != new_location.0 || end.1 != new_location.1 {
                return false;
            }
            end.0 += 1;
            return true;
        }
        // We can merge consecutive characters into a string
        else if let Some(Command::DeleteChar { location }) = self.undo_commands.last_mut() {
            let c = model.get_char_at(*location);
            // Can only merge consecutive alphabetic characters
            if !c.is_alphabetic() {
                return false;
            }
            if location.0 != new_location.0 || location.1 != new_location.1 {
                return false;
            }
            let cmd = Command::DeleteString{ start: (location.0 - 1, location.1), end: (location.0 + 1, location.1) };
            self.undo_commands.pop();
            self.undo_commands.push(cmd);
            return true;
        } 
        // We cannot merge any other commands
        else {
            return false;
        }
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
            
            self.change_count -= 1;
        }
    }

    pub fn execute_redo(&mut self, model: &mut Model) {
        if let Some(len) = self.redo_steps.pop() {
            for _ in 0..len {
                let cmd = self.redo_commands.pop().unwrap();
                let undo_cmd = cmd.execute(model);
                self.undo_commands.push(undo_cmd);
            }
            self.undo_steps.push(len);

            self.change_count += 1;
        }
    }

    pub fn reset_change_count(&mut self) {
        self.change_count = 0;
    }
}
