use console::style;

pub enum Level {
    ProgramOutput,
    Logging,
    Error,
    Warn,
    Selection
}

pub fn display_message(
    indent_level: Level, 
    message: &str
) {
    match indent_level {
        Level::Logging => println!(">> {}", style(message).green()),
        Level::Error => println!(">> {}", style(message).red().bold()),
        Level::ProgramOutput => println!(">> >> {}", style(message).cyan()),
        Level::Warn => println!(">> {}", style(message).red()),
        Level::Selection => println!(">> >> {}", style(message).blue())
    }
}