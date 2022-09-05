#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

pub mod term;
pub mod ansi_escaper;

#[cfg(test)]
mod tests {
    use alloc::vec;
    use std::println;
    use crate::ansi_escaper;
    use crate::ansi_escaper::{AnsiType, CSIType};

    #[test]
    fn incomplete_ansi() {
        let incomplete = ansi_escaper::escape("\x1Bm");
        assert_eq!(incomplete.0, AnsiType::Incomplete);
        assert_eq!(incomplete.1, 0);
    }

    #[test]
    fn simple_color() {
        let incomplete = ansi_escaper::escape("\x1B[1;1H hello");
        println!();
        assert_eq!(incomplete.0, AnsiType::CSI { kind: CSIType::SGR(0, vec![0]) });
        assert_eq!(incomplete.1, 0);
    }
}
