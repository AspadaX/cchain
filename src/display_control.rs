use console::style;
use prettytable::{Cell, Row, Table};

thread_local! {
    static DEPTH: std::cell::Cell<usize> = std::cell::Cell::new(1);
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

#[derive(Debug, Clone, Copy)]
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

pub fn display_form(column_labels: Vec<&str>, rows: Vec<Vec<&str>>) {
    let mut table = Table::new();
    let top_line: Vec<Cell> = column_labels
        .iter()
        .map(|item| Cell::new(item))
        .collect();
    table.add_row(
        Row::new(top_line)
    );

    for row in rows {
        table.add_row(
            Row::new(
                row.iter()
                .map(|item| Cell::new(item))
                .collect()
            )
        );
    }

    table.printstd();
}