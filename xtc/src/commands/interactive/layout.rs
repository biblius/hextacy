use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

// General layout
pub const HEADER: Constraint = Constraint::Percentage(10);
pub const BODY: Constraint = Constraint::Percentage(80);
pub const FOOTER: Constraint = Constraint::Percentage(10);

pub const VERTICAL_LAYOUT: &[Constraint] = &[HEADER, BODY, FOOTER];

pub const COMMAND_BOX: Constraint = Constraint::Percentage(33);
pub const SUBCOMMAND_BOX: Constraint = Constraint::Percentage(33);
pub const OPTION_BOX: Constraint = Constraint::Percentage(34);

pub const HORIZONTAL_LAYOUT: &[Constraint] = &[COMMAND_BOX, SUBCOMMAND_BOX, OPTION_BOX];

pub fn vertical<B: Backend>(frame: &Frame<B>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(VERTICAL_LAYOUT)
        .split(frame.size())
}

pub fn horizontal(body: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(HORIZONTAL_LAYOUT)
        .split(body)
}
