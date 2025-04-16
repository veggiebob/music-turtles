/*

Grammar := `start ` NonTerminal `\n` Production*

Production := NonTerminal `=` MusicString

MusicString := MusicPrimitive*

MusicPrimitive :=
  | Symbol
  | `{` (MusicString `|`)* MusicString? `}`
  | `[` usize `][` MusicString `]`

Symbol :=
  | NonTerminal
  | `:` Terminal

NonTerminal := [-a-zA-Z]

Terminal :=
  | Note (`<` Duration `>`)?
  | `:` MetaControl

Note :=
  | `_`
  | Int?[a-gA-G](b|#)?

MetaControl :=
  | `i=` Instrument
  | `v=` Volume

Instrument := Sine | piano | ...

Volume := Int

------ Examples --------

```
start S
S = [3][:4c<1> :4d :_ :f# :g :c ::i=piano B]
B = :0c
```

*/
use crate::cfg::{Grammar, MusicPrimitive, MusicString, NonTerminal, Symbol};

pub enum ScanError {
    Generic(String),
    ExpectedEither(String, String),
}

pub type Result<T> = std::result::Result<T, ScanError>;

pub trait Scanner {
    type Output;
    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)>;
}

type ScanPrefix = String;

pub struct GrammarScanner;

pub struct ProductionScanner;

pub struct MusicStringScanner;

pub struct MusicPrimitiveScanner;

pub struct SymbolScanner;

pub struct NonTerminalScanner;

pub struct TerminalScanner;

pub struct TerminalNoteScanner;

pub struct MetaControlScanner;

pub struct InstrumentScanner;

pub struct VolumeScanner;

impl Scanner for GrammarScanner {
    type Output = Grammar;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        concat(
            scan_map(concat(
                StringScanner("start".to_string()),
                scan_map(concat(
                    SpaceScanner,
                    NonTerminalScanner,
                ), |(_s, nt)| nt)
            ), |(_s, nt)| nt),
            kleene(ProductionScanner)
        ).scan(input)
            .map(|((s, prod), left)| (Grammar {
                start: NonTerminal::Custom(s),
                productions: prod,
            }, left))
    }
}

impl Scanner for ProductionScanner {
    type Output = (NonTerminal, MusicString);
    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        concat(
            scan_map(concat(
                NonTerminalScanner,
                StringScanner("=".to_string()),
            ), |(nt, _s)| NonTerminal::Custom(nt)),
            MusicStringScanner
        ).scan(input)
            .map(|((nt, music_string), left)| ((nt, music_string), left))
    }
}

impl Scanner for MusicStringScanner {
    type Output = MusicString;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut music_string = Vec::new();
        let mut remaining_input = input;

        while !remaining_input.is_empty() {
            if let Ok((primitive, new_input)) = MusicPrimitiveScanner.scan(remaining_input) {
                music_string.push(primitive);
                remaining_input = new_input;
            } else {
                break;
            }
        }

        Ok((MusicString(music_string), remaining_input))
    }
}

impl Scanner for MusicPrimitiveScanner {
    type Output = MusicPrimitive;
    
    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        todo!()
    }
}

impl Scanner for SymbolScanner {
    type Output = NonTerminal;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first.is_alphabetic() {
                let mut nt = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '_' {
                        nt.push(c);
                    } else {
                        return Ok((NonTerminal::Custom(nt), chars.as_str()));
                    }
                }
                return Ok((NonTerminal::Custom(nt), chars.as_str()));
            }
        }
        Err(ScanError::Generic("Expected NonTerminal".to_string()))
    }
}

impl Scanner for TerminalScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first == ':' {
                let mut terminal = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '_' {
                        terminal.push(c);
                    } else {
                        return Ok((terminal, chars.as_str()));
                    }
                }
                return Ok((terminal, chars.as_str()));
            }
        }
        Err(ScanError::Generic("Expected Terminal".to_string()))
    }
}

impl Scanner for TerminalNoteScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first == '_' || first.is_alphabetic() {
                let mut note = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '#' || c == 'b' {
                        note.push(c);
                    } else {
                        return Ok((note, chars.as_str()));
                    }
                }
                return Ok((note, chars.as_str()));
            }
        }
        Err(ScanError::Generic("Expected TerminalNote".to_string()))
    }
}

impl Scanner for MetaControlScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first == 'i' || first == 'v' {
                let mut meta_control = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '=' {
                        meta_control.push(c);
                    } else {
                        return Ok((meta_control, chars.as_str()));
                    }
                }
                return Ok((meta_control, chars.as_str()));
            }
        }
        Err(ScanError::Generic("Expected MetaControl".to_string()))
    }
}

impl Scanner for NonTerminalScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first.is_alphabetic() {
                let mut nt = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '_' {
                        nt.push(c);
                    } else {
                        return Ok((nt, chars.as_str()));
                    }
                }
                return Ok((nt, chars.as_str()));
            }
        }
        Err(ScanError::Generic("Expected NonTerminal".to_string()))
    }
}

pub struct StringScanner(String);

impl Scanner for StringScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        if input.starts_with(&self.0) {
            Ok((self.0.clone(), &input[self.0.len()..]))
        } else {
            Err(ScanError::Generic(format!(
                "Expected string: {}",
                self.0
            )))
        }
    }
}

pub struct SpaceScanner;

impl Scanner for SpaceScanner {
    type Output = ();

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let trimmed = input.trim_start();
        if trimmed.len() < input.len() {
            Ok(((), trimmed))
        } else {
            Err(ScanError::Generic("Expected space".to_string()))
        }
    }
}


pub struct ConcatScan<S, T>(S, T);
pub struct DisjointScan<S, T> {
    scanner_a: (ScanPrefix, S),
    scanner_b: (Option<ScanPrefix>, T),
}

pub struct KleeneScan<S>(S);

pub struct MapScanner<S, F> {
    scanner: S,
    mapper: F,
}

pub fn scan_map<S, F, T>(scan: S, map: F) -> impl Scanner<Output = T>
where
    S: Scanner,
    F: Fn(S::Output) -> T,
{
    MapScanner { scanner: scan, mapper: map }
}

pub fn kleene<S>(scan: S) -> impl Scanner<Output = Vec<S::Output>>
where
    S: Scanner,
{
    KleeneScan(scan)
}

pub fn concat<S, T, U, V>(scan1: S, scan2: T) -> impl Scanner<Output = (U, V)>
where
    S: Scanner<Output = U>,
    T: Scanner<Output = V>,
{
    ConcatScan(scan1, scan2)
}

pub fn disjoint<S, T, U>(
    prefix1: ScanPrefix,
    scan1: S,
    prefix2: Option<ScanPrefix>,
    scan2: T,
) -> impl Scanner<Output = U>
where
    S: Scanner<Output = U>,
    T: Scanner<Output = U>,
{
    DisjointScan {
        scanner_a: (prefix1, scan1),
        scanner_b: (prefix2, scan2),
    }
}

impl<S, T, U, V> Scanner for ConcatScan<S, T>
where
    S: Scanner<Output = U>,
    T: Scanner<Output = V>,
{
    type Output = (U, V);

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        self.0
            .scan(input)
            .and_then(|(u, new_input)| self.1.scan(new_input).map(|(v, s)| ((u, v), s)))
    }
}

impl<S, T, U> Scanner for DisjointScan<S, T>
where
    S: Scanner<Output = U>,
    T: Scanner<Output = U>,
{
    type Output = U;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        if input.starts_with(&self.scanner_a.0) {
            self.scanner_a.1.scan(input)
        } else if let Some(prefix) = &self.scanner_b.0 {
            if input.starts_with(prefix) {
                self.scanner_b.1.scan(input)
            } else {
                Err(ScanError::ExpectedEither(
                    self.scanner_a.0.to_string(),
                    self.scanner_b
                        .0
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or("Something else".to_string()),
                ))
            }
        } else {
            self.scanner_b.1.scan(input)
        }
    }
}


impl <S> Scanner for KleeneScan<S>
where
    S: Scanner,
{
    type Output = Vec<S::Output>;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut results = Vec::new();
        let mut remaining_input = input;

        while let Ok((result, new_input)) = self.0.scan(remaining_input) {
            results.push(result);
            remaining_input = new_input;
        }

        Ok((results, remaining_input))
    }
}

impl <S, T, U> Scanner for MapScanner<S, T>
where
    S: Scanner,
    T: Fn(S::Output) -> U,
{
    type Output = U;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        self.scanner.scan(input).map(|(output, new_input)| {
            ((self.mapper)(output), new_input)
        })
    }
}