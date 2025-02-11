use std::cell::Cell;

use console::style;

thread_local! {
    static DEPTH: Cell<usize> = Cell::new(1);
}

pub struct DepthGuard;

impl DepthGuard {
    pub fn enter() -> Self {
        DEPTH.with(
            |depth| 
            depth.set(depth.get() + 1)
        );
        Self
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        DEPTH.with(
            |depth|
            depth.set(
                depth.get() - 1
            )
        )
    }
}

pub enum Level {
    ProgramOutput,
    Logging,
    Error,
    Warn,
    Selection
}

pub fn display_message(
    level: Level, 
    message: &str
) {
    let depth: usize = DEPTH.with(
        |depth_variable| 
        depth_variable.get()
    );
    let indentation: String = ">> ".repeat(depth);

    match level {
        Level::Logging => println!("{}{}", indentation, style(message).green()),
        Level::Error => println!("{}{}", indentation, style(message).red().bold()),
        Level::ProgramOutput => println!("{}{}", indentation, style(message).cyan()),
        Level::Warn => println!("{}{}", indentation, style(message).red()),
        Level::Selection => println!("{}{}", indentation, style(message).blue())
    }
}