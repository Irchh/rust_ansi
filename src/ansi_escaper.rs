use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use core::fmt::{Display, Error, Formatter};
use core::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug, PartialEq)]
pub enum AnsiType {
    /// Normal text
    Text(String),

    /// Single Shift 2
    SS2,
    /// Single Shift 3
    SS3,
    /// Device Control String
    DCS,
    /// Control Sequence Introducer
    CSI {kind: CSIType},
    /// String Terminator
    ST,
    /// Operating System Command
    OSC {kind: OSCType},
    /// Reset to Initial State
    RIS,

    // These three can be ignored (after parsing), as they are usually application specific
    /// Start of String
    SOS,
    /// Privacy Message
    PM,
    /// Application Program Command
    APC,

    /// Ansi sequence is not complete / has errors
    Incomplete,

    Unknown(String),
}

impl From<char> for AnsiType {
    fn from(ch: char) -> Self {
        match ch {
            'N' =>  { AnsiType::SS2 }
            'O' =>  { AnsiType::SS3 }
            'P' =>  { AnsiType::DCS }
            '[' =>  { AnsiType::CSI { kind: CSIType::Unknown(String::new()) } }
            '\\' => { AnsiType::ST }
            ']' =>  { AnsiType::OSC { kind: OSCType::Unknown(String::new()) } }
            'X' =>  { AnsiType::SOS }
            '*' =>  { AnsiType::PM }
            '_' =>  { AnsiType::APC }
            'c' =>  { AnsiType::RIS }
            _ => { AnsiType::Unknown(String::from(format!("Unknown ansi escape char: {}", ch))) }
        }
    }
}

impl From<&str> for AnsiType {
    fn from(gr: &str) -> Self {
        match gr {
            "N" =>  { AnsiType::SS2 }
            "O" =>  { AnsiType::SS3 }
            "P" =>  { AnsiType::DCS }
            "[" =>  { AnsiType::CSI { kind: CSIType::Unknown(String::new()) } }
            "\\" => { AnsiType::ST }
            "]" =>  { AnsiType::OSC { kind: OSCType::Unknown(String::new()) } }
            "X" =>  { AnsiType::SOS }
            "*" =>  { AnsiType::PM }
            "_" =>  { AnsiType::APC }
            "c" =>  { AnsiType::RIS }
            _ => { AnsiType::Unknown(String::from(format!("Unknown ansi escape char: {}", gr))) }
        }
    }
}

impl AnsiType {
    pub fn finish(gr: &str, t: AnsiType, args: Vec<String>) -> AnsiType {
        match t {
            AnsiType::SS2 => {AnsiType::ST}
            AnsiType::SS3 => {AnsiType::ST}
            AnsiType::DCS => {AnsiType::ST}
            AnsiType::CSI { .. } => {
                let csi = AnsiType::CSI { kind: CSIType::from(gr, args) };
                csi
            }
            AnsiType::ST => {AnsiType::ST}
            AnsiType::OSC { .. } => {AnsiType::OSC {kind: OSCType::from(gr, args)}}
            AnsiType::RIS => {AnsiType::ST}
            AnsiType::SOS => {AnsiType::ST}
            AnsiType::PM => {AnsiType::ST}
            AnsiType::APC => {AnsiType::ST}
            AnsiType::Incomplete => {AnsiType::ST}
            AnsiType::Unknown(s) => {AnsiType::Unknown(s)}
            AnsiType::Text(s) => {AnsiType::Text(s)}
        }
    }

    pub fn finish_grapheme(gr: &str, t: AnsiType, args: Vec<String>) -> AnsiType {
        match t {
            AnsiType::SS2 => {AnsiType::ST}
            AnsiType::SS3 => {AnsiType::ST}
            AnsiType::DCS => {AnsiType::ST}
            AnsiType::CSI { .. } => {
                let csi = AnsiType::CSI { kind: CSIType::from_grapheme(gr, args) };
                csi
            }
            AnsiType::ST => {AnsiType::ST}
            AnsiType::OSC { .. } => {AnsiType::OSC {kind: OSCType::from_grapheme(gr, args)}}
            AnsiType::RIS => {AnsiType::ST}
            AnsiType::SOS => {AnsiType::ST}
            AnsiType::PM => {AnsiType::ST}
            AnsiType::APC => {AnsiType::ST}
            AnsiType::Incomplete => {AnsiType::ST}
            AnsiType::Unknown(s) => {AnsiType::Unknown(s)}
            AnsiType::Text(s) => {AnsiType::Text(s)}
        }
    }

    pub fn valid_char_ranges(t: &AnsiType) -> (Range<u32>, Range<u32>) {
        let mut end_char_range = 1..0;
        (match t {
            AnsiType::Text(_) => {1..0}
            AnsiType::SS2 => {1..0}
            AnsiType::SS3 => {1..0}
            AnsiType::DCS => {1..0}
            AnsiType::CSI { .. } => {end_char_range = 0x40..0x80; 0x20..0x40}
            AnsiType::ST => {1..0}
            AnsiType::OSC { .. } => {end_char_range = 0x7..0x8; 0x20..0x80}
            AnsiType::RIS => {1..0}
            AnsiType::SOS => {1..0}
            AnsiType::PM => {1..0}
            AnsiType::APC => {1..0}
            AnsiType::Incomplete => {1..0}
            AnsiType::Unknown(_) => {1..0}
        }, end_char_range)
    }
}

impl Display for AnsiType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let _ = match self {
            AnsiType::Text(s) => f.write_str(format!("Text({})", s).as_str()),
            AnsiType::SS2 => {f.write_str("SS2")}
            AnsiType::SS3 => {f.write_str("SS3")}
            AnsiType::DCS => {f.write_str("DCS")}
            AnsiType::CSI { kind } => {
                let _ = match kind {
                    CSIType::CUU(n) => {
                        f.write_str(format!("CUU {{ n: {}", n).as_str())
                    }
                    CSIType::CUD(n) => {
                        f.write_str(format!("CUD {{ n: {}", n).as_str())
                    }
                    CSIType::CUF(n) => {f.write_str(format!("CUF {{ n: {}", n).as_str())}
                    CSIType::CUB(n) => {f.write_str(format!("CUB {{ n: {}", n).as_str())}
                    CSIType::CNL(n) => {f.write_str(format!("CNL {{ n: {}", n).as_str())}
                    CSIType::CPL(n) => {f.write_str(format!("CPL {{ n: {}", n).as_str())}
                    CSIType::CHA(n) => {f.write_str(format!("CHA {{ n: {}", n).as_str())}
                    CSIType::CVA(n) => {f.write_str(format!("CVA {{ n: {}", n).as_str())}
                    CSIType::CUP(n, m) => {f.write_str(format!("CUP {{ n: {}, m: {}", n, m).as_str())}
                    CSIType::ED(n) => {f.write_str(format!("ED {{ n: {}", n).as_str())}
                    CSIType::EL(n) => {f.write_str(format!("EL {{ n: {}", n).as_str())}
                    CSIType::SU(n) => {f.write_str(format!("SU {{ n: {}", n).as_str())}
                    CSIType::SD(n) => {f.write_str(format!("SD {{ n: {}", n).as_str())}
                    CSIType::IL(n) => {f.write_str(format!("IL {{ n: {}", n).as_str())}
                    CSIType::HVP(n, m) => {f.write_str(format!("HVP {{ n: {}, m: {}", n, m).as_str())}
                    CSIType::SGR(n, m) => {f.write_str(format!("SGR {{ n: {}, m: {:?}", n, m).as_str())}
                    CSIType::DECSTBM(n, m) => {f.write_str(format!("DECSTBM {{ n: {}, m: {:?}", n, m).as_str())}
                    CSIType::DECSLRM(n, m) => {f.write_str(format!("DECSLRM {{ n: {}, m: {:?}", n, m).as_str())}
                    CSIType::DECTCEM(h) => {f.write_str(format!("DECTCEM {{ h: {:?}", h).as_str())}
                    CSIType::Unknown(s) => {f.write_str(format!("CSI {{ Unknown: {:?}", s).as_str())}
                };
                f.write_str(" }")
            } // End CSI

            AnsiType::ST => {f.write_str("ST")}
            AnsiType::OSC { kind } => {
                let _ = match kind {
                    OSCType::WindowTitle(s) => {f.write_str(format!("OSC {{ WindowTitle: {:?}", s).as_str())}
                    OSCType::Unknown(s) => {f.write_str(format!("OSC {{ Unknown: {:?}", s).as_str())}
                };
                f.write_str(" }")
            }
            AnsiType::RIS => {f.write_str("RIS")}
            AnsiType::SOS => {f.write_str("SOS")}
            AnsiType::PM => {f.write_str("PM")}
            AnsiType::APC => {f.write_str("APC")}
            AnsiType::Unknown(s) => {f.write_str(format!("Unknown: {:?}", s).as_str())}
            AnsiType::Incomplete => {f.write_str("Incomplete")}
        };
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OSCType {
    WindowTitle(String),
    Unknown(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum CSIType {
    // Cursor manipulation
    CUU(usize),
    CUD(usize),
    CUF(usize),
    CUB(usize),
    CNL(usize),
    CPL(usize),
    CHA(usize),
    CVA(usize),
    CUP(usize,usize),

    ED(usize),
    EL(usize),

    SU(usize),
    SD(usize),

    IL(usize),

    HVP(usize,usize),

    SGR(usize, Vec<usize>),

    DECTCEM(bool),
    DECSTBM(usize, usize),
    DECSLRM(usize, usize),

    Unknown(String),
}

impl OSCType {
    pub fn from(gr: &str, args: Vec<String>) -> OSCType {
        match args[0].as_str() {
            "0" => /* BEL */ {
                OSCType::WindowTitle(args[1].clone())
            }
            _ => { OSCType::Unknown(String::from(format!("Unknown OSC command: {:?}", gr)))}
        }
    }

    pub fn from_grapheme(gr: &str, args: Vec<String>) -> OSCType {
        match args[0].as_str() {
            "0" => /* BEL */ {
                OSCType::WindowTitle(args[1].clone())
            }
            _ => { OSCType::Unknown(String::from(format!("Unknown OSC command: {:?}", gr)))}
        }
    }
}

impl CSIType {
    pub fn from_grapheme(gr: &str, args: Vec<String>) -> CSIType {
        if gr.len() != 1 {
            CSIType::Unknown(format!("Unknown CSI command: {}", gr))
        } else {
            Self::from(gr, args)
        }
    }

    pub fn from(gr: &str, _args: Vec<String>) -> CSIType {
        let mut args = _args.clone();
        let mut private = false;
        if args[0].starts_with("?") {
            args[0].remove(0);
            private = true;
        }

        let first_arg_result = args[0].as_str().parse::<usize>();
        let n;
        let mut default = false;
        if first_arg_result.is_ok() {
            n = first_arg_result.unwrap();
        } else {
            n = 1;
            default = true;
        }

        let m;
        if args.len() > 1 {
            let m_res = args[1].as_str().parse::<usize>();
            if m_res.is_ok() {
                m = m_res.unwrap();
            } else {
                m = 1;
            }
        } else {
            m = 1;
        }

        if !private {
            match gr {
                "A" => { CSIType::CUU(n) }
                "B" => { CSIType::CUD(n) }
                "C" => { CSIType::CUF(n) }
                "D" => { CSIType::CUB(n) }
                "E" => { CSIType::CNL(n) }
                "F" => { CSIType::CPL(n) }
                "G" => { CSIType::CHA(n) }
                "d" => { CSIType::CVA(n) }
                "H" => { CSIType::CUP(n, m) }
                "J" => { CSIType::ED( if default {0} else {n} ) }
                "K" => { CSIType::EL( if default {0} else {n} ) }
                "L" => { CSIType::IL(n) }
                "S" => { CSIType::SU(n) }
                "T" => { CSIType::SD(n) }
                "f" => { CSIType::CUP(n, m) }
                "m" => {
                    let mut sgr_args = Vec::<usize>::new();
                    for i in 1..args.len() {
                        let res = args[i].as_str().parse::<usize>();
                        if res.is_ok() {
                            sgr_args.push(res.unwrap());
                        } else {
                            sgr_args.push(0);
                        }
                    }
                    CSIType::SGR(if default {0} else {n}, sgr_args)
                }
                "r" => { CSIType::DECSTBM(n, m) }
                "s" => { CSIType::DECSLRM(n, m) }
                _ => { CSIType::Unknown(format!("Unknown CSI command: {}", gr)) }
            }
        } else {
            match n {
                25 => {
                    match gr {
                        "h" => { CSIType::DECTCEM(true) }
                        "l" => { CSIType::DECTCEM(false) }
                        _ => { CSIType::Unknown(format!("Unknown Private CSI command: {}{}", n, gr))}
                    }
                }
                _ => { CSIType::Unknown(format!("Unknown Private CSI command: {}", n)) }
            }
        }
    }
}

pub struct AnsiEscaper {
    graphemes: Vec<String>,
}

impl Iterator for AnsiEscaper {
    type Item = AnsiType;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.parse_next())
    }
}

impl AnsiEscaper {
    pub const fn new() -> Self {
        Self {
            graphemes: vec![],
        }
    }

    pub fn new_text<S: AsRef<str>>(&mut self, str: S) {
        let new_graphemes = str.as_ref().graphemes(false).collect::<Vec<&str>>();
        for gr in new_graphemes {
            self.graphemes.push(String::from(gr));
        }
    }

    /// Returns the next ANSI code or next normal string, whichever is first.
    pub fn parse_next(&mut self) -> AnsiType {
        let mut string = String::new();
        while let Some(gr) = self.graphemes.first() {
            if gr == "\x1B" {
                if string.is_empty() {
                    return self.parse();
                } else {
                    return AnsiType::Text(string);
                }
            }
            string += gr;
            self.graphemes.remove(0);
        }

        AnsiType::Incomplete
    }

    fn next_grapheme(&mut self) -> Option<String> {
        let mut ret = None;
        if let Some(pog) = self.graphemes.first() {
            ret = Some(pog.clone());
            self.graphemes.remove(0);
        }
        ret
    }

    fn parse(&mut self) -> AnsiType {
        if self.graphemes.first() == Some(&String::from("\x1B"))  {
            self.graphemes.remove(0);
        }

        let ansi_type = AnsiType::from(self.next_grapheme().unwrap().as_str());
        match ansi_type {
            AnsiType::Text(_) => {}
            AnsiType::SS2 => {}
            AnsiType::SS3 => {}
            AnsiType::DCS => {}
            AnsiType::CSI { .. } => {}
            AnsiType::ST => {}
            AnsiType::OSC { .. } => {}
            AnsiType::RIS => {}
            AnsiType::SOS => {}
            AnsiType::PM => {}
            AnsiType::APC => {}
            AnsiType::Incomplete => {}
            AnsiType::Unknown(_) => {}
        }

        AnsiType::Incomplete
    }
}

pub trait ToAnsi {
    fn to_ansi(&self) -> AnsiEscaper;
}

impl ToAnsi for &str {
    fn to_ansi(&self) -> AnsiEscaper {
        let mut escaper = AnsiEscaper::new();
        escaper.new_text(self);
        escaper
    }
}

pub fn read_until_escape_char<S: AsRef<str>>(s: S) -> String {
    let graphemes = s.as_ref().graphemes(false).collect::<Vec<&str>>();

    let mut string = String::new();

    for grapheme in graphemes {
        if grapheme == "\x1B" {
            break;
        }
        string += grapheme;
    }

    string
}

/// Escapes a given string, and returns the first found ANSI code and how many characters it occupies in a tuple.
pub fn escape<S: AsRef<str>>(s: S) -> (AnsiType, usize) {
    let graphemes = s.as_ref().graphemes(false).collect::<Vec<&str>>();

    if graphemes.len() == 0 {
        return (AnsiType::Incomplete,0);
    }
    if graphemes.len() < 2 || graphemes[0] != "\x1B" /* Escape char */ {
        let string = read_until_escape_char(s);
        let length = string.len();
        return (AnsiType::Text(string), length);
        //return (AnsiType::Unknown(String::from("First character not escape char")),1);
    }
    if graphemes[1] == ">" {
        return (AnsiType::Unknown(String::from("I do not know how to handle '>'")),2);
    }
    if graphemes.len() < 3 {
        return (AnsiType::Incomplete, 0);
    }

    let t = AnsiType::from(graphemes[1]);

    let char_ranges = AnsiType::valid_char_ranges(&t);
    //let mut special = false;
    match t {
        AnsiType::CSI { .. } => {
            // TODO: Handle special (OEM) CSI codes
            /*if byte_arr[2] != '?' as u8 {
                special = true;
            }*/
        }
        AnsiType::Unknown(e_str) => {return (AnsiType::Unknown(e_str),2)}
        _ => {}
    }

    let valid_char_ranges = char_ranges.0;
    let end_char_range= 1..0;

    let mut arguments: Vec<String> = Vec::new();
    let mut curr_arg = String::new();
    let mut i = 0;
    let mut escaping = false;
    let mut ansi_string = String::new();

    for grapheme in graphemes {
        if i < 2 { i += 1; continue; }
        i += 1;
        if grapheme.len() > 1 {
            return (AnsiType::Unknown(String::new()), 0);
        }

        if grapheme == "\x1b" || escaping {
            escaping = true;
            ansi_string += grapheme.clone();
            let res = crate::ansi_escaper::escape(ansi_string.clone());
            if res.1 > 0 {
                match res.0 {
                    AnsiType::ST => {
                        return (AnsiType::finish("\x07", t, arguments),i);
                    }
                    _ => {
                    }
                }
                escaping = false;
            }
            continue;
        }

        if grapheme == ";" {
            arguments.push(curr_arg.clone());
            curr_arg.clear();
            continue;
        }

        let ch = grapheme.as_bytes()[0] as char;

        if valid_char_ranges.contains(&u32::from(ch)) {
            curr_arg.push(ch);
        } else if end_char_range.contains(&u32::from(ch)) {
            arguments.push(curr_arg.clone());
            return (AnsiType::finish(grapheme, t, arguments), i);
            // Get CSI Type
        } else {
            arguments.push(curr_arg.clone());
            return (AnsiType::finish(grapheme, t, arguments), i);
            //return (AnsiType::Unknown(format!("Illegal character {:?} found in escape sequence", ch)), i);
        }
    }

    (AnsiType::Incomplete, 0)
}