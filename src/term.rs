// TODO: Extract this into a new crate
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use crate::ansi_escaper::{AnsiEscaper, AnsiType, CSIType, OSCType};

extern crate unicode_segmentation;

pub trait TermInterface<T> {
    /// Returns a reference to the imaginary framebuffer of the terminal.
    // TODO: Is this good enough? Is there a need for a mutable reference?
    fn framebuffer(&self) -> &T;
    /// Called when renderer has rendered so you can check for dirty bits.
    // TODO: Remove and force rendering side to buffer the framebuffer maybe?
    fn completed_render(&mut self);

    /// Write plain text to screen. `s` should not contain any ANSI codes.
    fn write(&mut self, s: String);
    /// Moves cursor absolute X position. Top left of the screen is 1,1.
    fn goto_x(&mut self, x: usize);
    /// Moves cursor absolute Y position. Top left of the screen is 1,1.
    fn goto_y(&mut self, y: usize);
    /// Moves cursor relative X position. Top left of the screen is 1,1.
    fn move_x(&mut self, x: isize);
    /// Moves cursor relative Y position. Top left of the screen is 1,1.
    fn move_y(&mut self, y: isize);
    /// Gets called whenever I feel like it. Not neccesary
    fn draw(&mut self) {
        // Only needed if the implementer wants to buffer changes until necessary.
    }

    /// Moves cursor absolute X/Y position. Top left of the screen is 1,1.
    fn goto(&mut self, x: usize, y: usize) {
        self.goto_x(x);
        self.goto_y(y);
    }
    /// Moves cursor relative X/Y position. Top left of the screen is 1,1.
    fn goto_rel(&mut self, x: isize, y: isize) {
        self.move_x(x);
        self.move_y(y);
    }

    // CSI
    /// Moves the cursor up *n* (default `1`) cells. If the cursor is already at the edge of the screen, this has no effect.
    fn cursor_up(&mut self, n: usize) {
        self.goto_rel(0, -(n as isize));
    }
    /// Moves the cursor down *n* (default `1`) cells. If the cursor is already at the edge of the screen, this has no effect.
    fn cursor_down(&mut self, n: usize) {
        self.goto_rel(0, n as isize);
    }
    /// Moves the cursor forward *n* (default `1`) cells. If the cursor is already at the edge of the screen, this has no effect.
    fn cursor_forward(&mut self, n: usize) {
        self.goto_rel(n as isize, 0);
    }
    /// Moves the cursor back *n* (default `1`) cells. If the cursor is already at the edge of the screen, this has no effect.
    fn cursor_back(&mut self, n: usize) {
        self.goto_rel(-(n as isize), 0);
    }
    /// Moves the cursor to the beginning of the line *n* (default `1`) lines down.
    fn cursor_next_line(&mut self, n: usize) {
        self.goto_x(1);
        self.move_y(n as isize);
    }
    /// Moves the cursor to the beginning of the line *n* (default `1`) lines up.
    fn cursor_prev_line(&mut self, n: usize) {
        self.goto_x(1);
        self.move_y(-(n as isize));
    }
    /// Moves the cursor to column *n* (default `1`).
    fn cursor_horizontal_absolute(&mut self, n: usize) {
        self.goto_y(n);
    }
    /// Moves the cursor to row *n* (default `1`).
    fn cursor_vertical_absolute(&mut self, n: usize) {
        self.goto_x(n);
    }
    /// Moves the cursor to row *n*, column *m* (default `1`/`1`).
    fn cursor_position(&mut self, n: usize, m: usize) {
        self.goto(n, m);
    }
    /// Clears part of the screen.
    ///
    /// different `n`-values:
    ///
    /// - 0 - Clear from cursor to end of screen
    /// - 1 - Clear from cursor to beginning of screen
    /// - 2 - Clear entire screen
    /// - 3 - Clear entire screen, including scrollback buffer (if it is implemented)
    ///
    /// Note: Cursor position does not change.
    fn erase_in_display(&mut self, n: usize);
    /// Clears part of the line.
    ///
    /// different `n`-values:
    ///
    /// - 0 - Clear from cursor to end of line
    /// - 1 - Clear from cursor to beginning of line
    /// - 2 - Clear entire line
    ///
    /// Note: Cursor position does not change.
    fn erase_in_line(&mut self, n: usize);
    /// Scroll up page by `n` lines.
    fn scroll_up(&mut self, n: usize);
    /// Scroll down page by `n` lines.
    fn scroll_down(&mut self, n: usize);
    /// Moves the current line by `n` lines, clearing the current line in the process.
    // TODO: Rename function to more be intuitive.
    fn il(&mut self, n: usize);
    /// Moves the cursor to row *n*, column *m* (default `1`/`1`).
    fn horizontal_vertical_position(&mut self, n: usize, m: usize) {
        self.goto(n, m);
    }
    /// Sets colors and style of the characters following.
    fn select_graphics_rendition(&mut self, n: Vec<usize>);
    /// Set top and bottom margins. Moves the cursor to column 1, line 1 of the page.
    fn decstbm(&mut self, top: usize, bot: usize);
    /// Set left and right margins. Moves the cursor to column 1, line 1 of the page.
    fn decslrm(&mut self, left: usize, right: usize);
    /// Shows or hides cursor based on the `show` argument.
    fn dectcem(&mut self, show: bool);
    /// Should return a tuple of the current row and column as (row, column).
    fn device_status_report(&mut self) -> (usize, usize);
    /// Unknown csi code.
    fn unknown_csi(&mut self, s: String);

    // OSI
    /// Sets the title of the terminal window.
    fn set_title(&mut self, title: String);
    /// Unknown osc code.
    fn unknown_osc(&mut self, s: String);

    // Other
    /// Unknown ANSI code.
    fn unknown(&mut self, s: String);
}

pub struct Term<T> {
    ti: Box<dyn TermInterface<T>>,
    escaper: AnsiEscaper,
}

impl<T> Term<T> {
    pub const fn new(ti: Box<dyn TermInterface<T>>) -> Self {
        Self {
            ti,
            escaper: AnsiEscaper::new()
        }
    }

    /// Returns a reference to the imaginary framebuffer of the terminal.
    pub fn framebuffer(&self) -> &T {
        self.ti.framebuffer()
    }

    pub fn completed_render(&mut self) {
        self.ti.completed_render();
    }

    // TODO: what does this do?
    pub fn write<S: AsRef<str>>(&mut self, s: S) {
        self.escaper.new_text(s);
        loop {
            let ansi = self.escaper.parse_next();
            match ansi {AnsiType::Text(str) => self.ti.write(str),
                AnsiType::SS2 => {}
                AnsiType::SS3 => {}
                AnsiType::DCS => {}
                AnsiType::CSI { kind } => {
                    match kind {
                        CSIType::CUU(n) => self.ti.cursor_up(n),
                        CSIType::CUD(n) => self.ti.cursor_down(n),
                        CSIType::CUF(n) => self.ti.cursor_forward(n),
                        CSIType::CUB(n) => self.ti.cursor_back(n),
                        CSIType::CNL(n) => self.ti.cursor_next_line(n),
                        CSIType::CPL(n) => self.ti.cursor_prev_line(n),
                        CSIType::CHA(n) => self.ti.cursor_horizontal_absolute(n),
                        CSIType::CVA(n) => self.ti.cursor_vertical_absolute(n),
                        CSIType::CUP(n, m) => self.ti.cursor_position(n, m),
                        CSIType::ED(n) => self.ti.erase_in_display(n),
                        CSIType::EL(n) => self.ti.erase_in_line(n),
                        CSIType::SU(n) => self.ti.scroll_up(n),
                        CSIType::SD(n) => self.ti.scroll_down(n),
                        CSIType::IL(n) => self.ti.il(n),
                        CSIType::HVP(n, m) => self.ti.horizontal_vertical_position(n, m),
                        CSIType::SGR(n) => self.ti.select_graphics_rendition(n),
                        CSIType::DECSTBM(top, bot) => self.ti.decstbm(top, bot),
                        CSIType::DECSLRM(top, bot) => self.ti.decslrm(top, bot),
                        CSIType::DECTCEM(show) => self.ti.dectcem(show),
                        CSIType::Unknown(s) => self.ti.unknown_csi(s),
                    }
                }
                AnsiType::ST => {}
                AnsiType::OSC { kind } => {
                    match kind {
                        OSCType::WindowTitle(title) => self.ti.set_title(title),
                        OSCType::Unknown(s) => self.ti.unknown_osc(s),
                    }
                }
                AnsiType::RIS => {}
                AnsiType::SOS => {}
                AnsiType::PM => {}
                AnsiType::APC => {}
                AnsiType::Incomplete => {
                    break;
                }
                AnsiType::Unknown(str) => self.ti.unknown(str),
            }
        }
    }
}