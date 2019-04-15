use std::collections::HashMap;

use crate::core::*;

pub struct SyntaxParser<'a> {
    pointer: Option<usize>,
    source: Vec<String>,
    methods: HashMap<String, usize>,
    environment: &'a mut Karel,
}

impl<'a> SyntaxParser<'a> {
    /// Create new syntax parser. It takes list of strings that represent
    /// program soRsult<SyntaxParser, Sult<SyntaxParserurce. They are read and methods are found and indexed.
    /// One can then run program with `run` or `step`.
    pub fn new(sources: Vec<String>, environment: &'a mut Karel) -> SyntaxParser {
        let mut sp = SyntaxParser {
            pointer: None,
            source: SyntaxParser::preprocess(sources),
            methods: HashMap::new(),
            environment,
        };
        sp.index_methods();
        if sp.methods.contains_key("main") {
            sp.pointer = Some(sp.methods["main"]);
        }
        sp
    }

    /// Run method until the program ends or a an error is encountered.
    pub fn run(&mut self) -> Result<(), RuntimeError> {
        match self.pointer {
            None => Err(RuntimeError::NoEntryPointDefined),
            Some(_) => self.run_block(false),
        }
    }

    /// Run underlying block of code. If the code block is to be skipped
    /// (for example because conditional was false), setting `skip_block` to `true`
    /// will make this code just advance the pointer to block end.
    fn run_block(&mut self, skip_block: bool) -> Result<(), RuntimeError> {
        match self.pointer {
            None => Err(RuntimeError::RuntimeSyntaxError(
                SyntaxError::UnexpectedEndOfFile,
            )),
            Some(_) => {
                // Find out which type of block am I even running
                let command = SyntaxParser::find_command(&self.source[self.pointer.unwrap()]);
                let current_syntax_block: SyntaxBlock = match command.get(0) {
                    Some(&"if") => SyntaxBlock::If,
                    Some(&"def") => SyntaxBlock::Def,
                    Some(&"repeat") => SyntaxBlock::Repeat,
                    Some(&"while") => SyntaxBlock::While,
                    None => {
                        return Err(RuntimeError::RuntimeSyntaxError(
                            SyntaxError::ExpectedSomethingElse(
                                "Expected syntax block (if, def, repeat, while). Got: {}",
                            ),
                        ));
                    }
                    Some(_) => {
                        return Err(RuntimeError::RuntimeSyntaxError(SyntaxError::NotDefined(
                            &self.source[self.pointer.unwrap()],
                        )));
                    }
                };

                loop {
                    self.pointer = Some(self.pointer.unwrap() + 1);
                    if self.source.len() >= self.pointer.unwrap() {
                        return Err(RuntimeError::RuntimeSyntaxError(
                            SyntaxError::UnexpectedEndOfFile,
                        ));
                    }

                    let command: Vec<&str> = SyntaxParser::find_command(&self.source[self.pointer.unwrap()]);
                    // Check if we didn't end the block
                    let syntax_block_end: &str = get_syntax_block_end(&current_syntax_block);
                    match command.get(0) {
                        Some(text) if text == &syntax_block_end => {
                            // We reached end of current block. Advance pointer and return.
                            self.pointer = Some(self.pointer.unwrap() + 1);
                            return Ok(());
                        }
                        // Match any other command
                        Some(text) => {
                            if skip_block {
                                continue;
                            }

                            match text {
                                /* ACTIONS */
                                &"turn-left" => {
                                    // Move left
                                    let result = self.environment.action(Action::TurnLeft);
                                    if let Err(result_error) = result {
                                        return Err(RuntimeError::RuntimeActionError(result_error));
                                    }
                                }
                                &"move" => {
                                    // Move forward
                                    let result = self.environment.action(Action::Move);
                                    if let Err(result_error) = result {
                                        return Err(RuntimeError::RuntimeActionError(result_error));
                                    }
                                }
                                &"take" => {
                                    // Take item from current tile
                                    let result = self.environment.action(Action::RemoveItem);
                                    if let Err(result_error) = result {
                                        return Err(RuntimeError::RuntimeActionError(result_error));
                                    }
                                }
                                &"put" => {
                                    // Put an item on current tile
                                    let result = self.environment.action(Action::PlaceItem);
                                    if let Err(result_error) = result {
                                        return Err(RuntimeError::RuntimeActionError(result_error));
                                    }
                                }
                                &"die" => {
                                    // End current block
                                    return Ok(());
                                }
                                /* IF */
                                &"if" => {
                                    // Match second argument, execute query, and run another block recursively (and either skip or run it)
                                    match command.get(1) {
                                        Some(text) => {
                                            let success: bool = match text {
                                                &"wall" => {
                                                    let result = self
                                                        .environment
                                                        .query(Query::WallInFrontOfMe);
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                &"beeper" => {
                                                    let result =
                                                        self.environment.query(Query::ItemHere);
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                &"north" => {
                                                    let result = self
                                                        .environment
                                                        .query(Query::Direction(Direction::North));
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                &"south" => {
                                                    let result = self
                                                        .environment
                                                        .query(Query::Direction(Direction::South));
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                &"west" => {
                                                    let result = self
                                                        .environment
                                                        .query(Query::Direction(Direction::West));
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                &"east" => {
                                                    let result = self
                                                        .environment
                                                        .query(Query::Direction(Direction::East));
                                                    match result {
                                                        Err(result_error) => {
                                                            return Err(
                                                                RuntimeError::RuntimeQueryError(
                                                                    result_error,
                                                                ),
                                                            );
                                                        }
                                                        Ok(ok_result) => ok_result,
                                                    }
                                                }
                                                _ => {
                                                    return Err(RuntimeError::RuntimeSyntaxError(
                                                        SyntaxError::NotDefined(
                                                            &self.source[self.pointer.unwrap()],
                                                        ),
                                                    ));
                                                }
                                            };

                                            let block_result = self.run_block(!success);

                                            if let Err(block_error) = block_result {
                                                return Err(block_error);
                                            }
                                        }
                                        None => {
                                            return Err(RuntimeError::RuntimeSyntaxError(
                                                SyntaxError::NotDefined(&self.source[self.pointer.unwrap()]),
                                            ));
                                        }
                                    }
                                }
                                /* CALL */
                                &"call" => {
                                    // Match second argument, and call the function as another block.
                                    match command.get(1) {
                                        Some(text) => {
                                            if self.methods.contains_key(*text) {
                                                // Save the pointer location so we can return after the method finishes
                                                let old_pointer: usize = self.pointer.unwrap();

                                                let block_result = self.run_block(true);

                                                if let Err(block_error) = block_result {
                                                    return Err(block_error);
                                                }

                                                // And return back after calling the method
                                                self.pointer = Some(old_pointer + 1);
                                            }
                                        }
                                        None => {
                                            return Err(RuntimeError::RuntimeSyntaxError(
                                                SyntaxError::NotEnoughArguments(
                                                    &self.source[self.pointer.unwrap()],
                                                ),
                                            ));
                                        }
                                    }
                                }
                                /* REPEAT */
                                &"repeat" => {
                                    // Match second argument, try to parse it to number, and run repeat block N-times
                                    match command.get(1) {
                                        Some(text) => {
                                            let number = text.parse::<usize>();
                                        }
                                        None => {
                                            return Err(RuntimeError::RuntimeSyntaxError(
                                                SyntaxError::NotEnoughArguments(
                                                    &self.source[self.pointer.unwrap()],
                                                ),
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(RuntimeError::RuntimeSyntaxError(
                                        SyntaxError::NotDefined(&self.source[self.pointer.unwrap()]),
                                    ));
                                }
                            };
                        }
                        None => {
                            return Err(RuntimeError::RuntimeSyntaxError(
                                SyntaxError::UnexpectedEndOfFile,
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Run one command from user.
    //pub fn interactive(&mut self, command: String) -> Result<(), RuntimeError> {}

    /// Take list of source file contents and preprocess it - trimming
    /// whitespaces, removing comments and empty lines.
    ///
    /// Result si list of lines.
    fn preprocess<'b>(source_files_content: Vec<String>) -> Vec<String> {
        // TODO: I copy *EVERY* line of source code. This will be VERRRYYY slow.
        let mut lines: Vec<String> = Vec::new();
        for source_file in source_files_content {
            for line in source_file.lines() {
                // Remove comments
                let comment_char = line.find("#");
                let mut parsed_line: &str;
                if let Some(comment_char) = comment_char {
                    parsed_line = &line[0..comment_char];
                } else {
                    parsed_line = &line;
                }
                // Remove whitespaces
                parsed_line = parsed_line.trim();
                if parsed_line.len() != 0 {
                    lines.push(String::from(parsed_line));
                }
            }
        }

        lines
    }

    fn index_methods(&mut self) {
        let mut current_index: usize = 0;
        for line in &self.source {
            if line.len() > 3 && line.starts_with("def") {
                let method_name = line[3..].trim();
                self.methods
                    .insert(String::from(method_name), current_index);
            }
            current_index += 1;
        }
    }

    /// Parse a line of source code, and return command and it's parameters as vector of strings
    fn find_command(line: &str) -> Vec<&str> {
        let splitted_line = line.split_whitespace().filter(|s| s.len() > 0);
        splitted_line.collect()
    }
}

pub enum RuntimeError<'a> {
    /// Main was not found. Consider calling `interactive` instead
    NoEntryPointDefined,
    RuntimeActionError(crate::core::ActionError),
    RuntimeQueryError(crate::core::QueryError),
    RuntimeSyntaxError(SyntaxError<'a>),
}

pub enum SyntaxError<'a> {
    /// Method that was called is not defined
    /// (this method should be defined by user)
    MethodNotDefined(&'a str),
    /// Non-user defined structure that was called is not defined
    /// (such as conditions, loops, and Karel commands)
    NotDefined(&'a str),
    /// Wrong block end encountered. Make sure you didn't mix up
    /// `endif`, `enddef`, `endrepeat`, `endwhile`
    WrongBlockEnd(&'a str),
    /// Unexpected end of file encountered. Make sure you included
    /// `endif`, `enddef`, `endrepeat`, or `endwhile`.
    UnexpectedEndOfFile,
    /// A number was wanted, but it cannot be converted from string.
    /// This is typically when user uses `repeat`.
    NotANumber(&'a str),
    /// Something else was expected. This is probably interpreter issue.
    ExpectedSomethingElse(&'a str),
    /// Not enough arguments to execute, or wrong arguments. For example
    /// condition statement without actual condition.
    NotEnoughArguments(&'a str),
}

enum SyntaxBlock {
    Repeat,
    If,
    Def,
    While,
}

fn get_syntax_block_end(block: &SyntaxBlock) -> &'static str {
    match block {
        SyntaxBlock::Repeat => "endrepeat",
        SyntaxBlock::If => "endif",
        SyntaxBlock::Def => "enddef",
        SyntaxBlock::While => "endwhile",
    }
}