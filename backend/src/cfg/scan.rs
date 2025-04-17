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

use crate::cfg::{Grammar, MetaControl, MusicPrimitive, MusicString, NonTerminal, Production, Symbol, Terminal, TerminalNote};
use crate::composition::{Instrument, Octave, Pitch, Volume};
use crate::time::{Beat, MusicTime};


#[derive(Debug)]
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
pub struct MusicPrimitiveSplitScanner;
pub struct MusicPrimitiveRepeatScanner;

pub struct SymbolScanner;

pub struct NonTerminalScanner;

pub struct TerminalScanner;

pub struct NoteScanner;

pub struct DurationScanner;

pub struct MetaControlScanner;

pub struct InstrumentScanner;

pub struct VolumeScanner;

impl Scanner for GrammarScanner {
    type Output = Grammar;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let lines = input.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();
        if lines.is_empty() {
            return Err(ScanError::Generic("Expected at least one line".to_string()));
        }
        let start_line = lines[0];
        let start = start_line
            .strip_prefix("start ")
            .ok_or_else(|| ScanError::Generic("Expected 'start' at the beginning of the first line".to_string()))?;
        let start = NonTerminalScanner.scan(start)
            .map(|(nt, _s)| NonTerminal::Custom(nt))?;
        let productions = lines[1..]
            .iter()
            .map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return Ok(None);
                }
                let (prod, _s) = ProductionScanner.scan(line)?;
                Ok(Some(prod))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .filter_map(|x| x)
            .collect();
        Ok((Grammar { start, productions }, ""))
    }
}

impl Scanner for ProductionScanner {
    type Output = Production;
    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        scan_map(concat(
            scan_map(
                concat(NonTerminalScanner, trim(StringScanner("=".to_string()))),
                |(nt, _s)| NonTerminal::Custom(nt),
            ),
            MusicStringScanner,
        ), |(nt, str)| Production(nt, str))
            .scan(input)
    }
}

impl Scanner for MusicStringScanner {
    type Output = MusicString;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut music_string = Vec::new();
        let mut remaining_input = input;

        while !remaining_input.is_empty() {
            // skip to the first non-whitespace character
            remaining_input = remaining_input.trim_start();
            if remaining_input.is_empty() {
                break;
            }
            match MusicPrimitiveScanner.scan(remaining_input) {
                Ok((primitive, new_input)) => {
                    music_string.push(primitive);
                    remaining_input = new_input;
                }
                Err(e) => {
                    println!("Error scanning MusicString: {:?}. Remaining input: {remaining_input}", e);
                    break;
                }
            }
        }

        Ok((MusicString(music_string), remaining_input))
    }
}

impl Scanner for MusicPrimitiveScanner {
    type Output = MusicPrimitive;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // split scanner, or else repeat scanner, or else SymbolScanner
        disjoint(
            ScanPrefix::from("{".to_string()),
            MusicPrimitiveSplitScanner,
            None,
            disjoint(
                ScanPrefix::from("[".to_string()),
                MusicPrimitiveRepeatScanner,
                None,
                scan_map(SymbolScanner, |s| MusicPrimitive::Simple(s)),
            ),
        )
            .scan(input)
    }
}

impl Scanner for MusicPrimitiveSplitScanner {
    type Output = MusicPrimitive;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // if it starts with '{', then find the matching '}' and split on each '|'
        if let Some('{') = input.chars().next() {
            let rest = &input[1..];
            if let Some(end) = find_matching(rest, '{', '}') {
                let inner = &rest[..end];
                let mut parts = inner.split('|');
                let first_part = parts.next().unwrap_or("");
                let rest_parts: Vec<_> = parts.collect();
                let scanner = consume(MusicStringScanner);
                let (music_string, _consumed) = scanner.scan(first_part)?;
                let rest_music_strings: Vec<_> = rest_parts
                    .iter()
                    .map(|&s| MusicStringScanner.scan(s))
                    .try_fold(vec![music_string], |mut vec, res| {
                        let (music_string, _consumed) = res?;
                        vec.push(music_string);
                        Ok(vec)
                    })?;
                let rest = &rest[end + 1..];
                Ok((MusicPrimitive::Split { branches: rest_music_strings }, rest))
            } else {
                Err(ScanError::Generic("Expected '}'".to_string()))
            }
        } else {
            Err(ScanError::Generic("Expected '{'".to_string()))
        }
    }
}

impl Scanner for MusicPrimitiveRepeatScanner {
    type Output = MusicPrimitive;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // first scan '[' a positive integer, '][', then a MusicString, and finally ']'
        if let Some('[') = input.chars().next() {
            if let Some(repeat_num_end) = input.find(']') {
                let repeat_num = &input[1..repeat_num_end];
                if let Some('[') = &input[repeat_num_end + 1..].chars().next() {
                    let rest = &input[repeat_num_end + 2..];
                    if let Some(end_bracket) = find_matching(rest, '[', ']')
                    {
                        let music_string = &rest[..end_bracket];
                        let scanner = consume(MusicStringScanner);
                        let music_string = scanner.scan(music_string).map(|(ms, _empty)| ms)?;
                        let rest = &rest[end_bracket + 1..];
                        Ok((
                            MusicPrimitive::Repeat {
                                num: repeat_num.parse().unwrap(),
                                content: music_string,
                            },
                            rest,
                        ))
                    } else {
                        Err(ScanError::Generic("Expected ']'".to_string()))
                    }
                } else {
                    Err(ScanError::Generic("Expected '['".to_string()))
                }
            } else {
                Err(ScanError::Generic("Expected ']'".to_string()))
            }
        } else {
            Err(ScanError::Generic("Expected '['".to_string()))
        }
    }
}

impl Scanner for SymbolScanner {
    type Output = Symbol;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // if it starts with ':', use TerminalScanner
        // otherwise, use NonTerminalScanner
        disjoint(
            ScanPrefix::from(":".to_string()),
            scan_map(scan_map_input(TerminalScanner, |s| &s[1..]), |s| {
                Symbol::T(s)
            }),
            None,
            scan_map(NonTerminalScanner, |s| Symbol::NT(NonTerminal::Custom(s))),
        )
            .scan(input)
    }
}

impl Scanner for TerminalScanner {
    type Output = Terminal;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // if it starts with ':', then use MetaControlScanner
        // otherwise, use TerminalNoteScanner
        disjoint(
            ScanPrefix::from(":".to_string()),
            scan_map_input(scan_map(MetaControlScanner, |s| Terminal::Meta(s)), |s| &s[1..]),
            None,
            scan_map(concat(NoteScanner, DurationScanner), |(note, duration)| {
                Terminal::Music {
                    note: note,
                    duration: duration,
                }
            }),
        )
            .scan(input)
    }
}

impl Scanner for NoteScanner {
    type Output = TerminalNote;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        /*
        Note :=
          | `_`
          | Int?[a-gA-G](b|#)?
        */
        let mut chars = input.chars();
        let mut rest = input;
        let mut octave = 4;
        let mut note = 0;
        let mut consumed = 0;
        if let Some(first) = chars.next() {
            consumed += 1;
            let next = if first == '_' {
                return Ok((TerminalNote::Rest, chars.as_str()));
            } else if let Some(dig) = first.to_digit(10) {
                octave = dig as Octave;
                consumed += 1;
                chars.next()
            } else {
                Some(first)
            };
            if let Some(next) = next {
                if 'a' <= next.to_ascii_lowercase() && next.to_ascii_lowercase() <= 'g' {
                    match next.to_ascii_lowercase() {
                        'a' => note = 0,
                        'b' => note = 2,
                        'c' => note = 3,
                        'd' => note = 5,
                        'e' => note = 7,
                        'f' => note = 8,
                        'g' => note = 10,
                        _ => unreachable!(),
                    }
                    if let Some(next) = chars.next() {
                        if next == '#' {
                            note += 1;
                            consumed += 1;
                        } else if next == 'b' {
                            note -= 1;
                            consumed += 1;
                        }
                    }
                    Ok((TerminalNote::Note { pitch: Pitch(octave, note) }, &input[consumed..]))
                } else {
                    Err(ScanError::Generic(
                        format!("Expected Note: note name {next} is not a valid note."),
                    ))
                }
            } else {
                Err(ScanError::Generic(
                    format!("Expected letter [a-g] after octave number after {first}"),
                ))
            }
        } else {
            Err(ScanError::Generic(
                "Expected Note: octave number or note letter".to_string(),
            ))
        }
    }
}

impl Scanner for DurationScanner {
    type Output = MusicTime;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // if it starts with '<', then scan a duration
        if let Some('<') = input.chars().next() {
            if let Some(end) = find_matching(&input[1..], '<', '>') {
                let duration = &input[1..=end];
                let rest = &input[end + 2..];
                if duration.contains('/') {
                    // it's a ratio
                    let mut parts = duration.split('/');
                    match (parts.next().and_then(|s| s.parse().ok()), parts.next().and_then(|s| s.parse().ok())) {
                        (Some(num), Some(denom)) => {
                            Ok((MusicTime(0, Beat::new(num, denom)), rest))
                        }
                        _ => {
                            eprintln!("Unable to parse {duration} as duration. Defaulting to 1");
                            Ok((MusicTime::beats(1), rest))
                        }
                    }
                } else {
                    let duration_int = duration.parse::<u32>().unwrap_or(0);
                    Ok((MusicTime::beats(duration_int), rest))
                }
            } else {
                Err(ScanError::Generic("Expected '>'".to_string()))
            }
        } else {
            Ok((MusicTime::beats(1), input))
        }
    }
}

impl Scanner for MetaControlScanner {
    type Output = MetaControl;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if let Some('=') = chars.next() {
                let mut rest = &input[2..];
                match first {
                    'i' => {
                        let (instrument, new_input) = InstrumentScanner.scan(rest)?;
                        rest = new_input;
                        Ok((MetaControl::ChangeInstrument(instrument), rest))
                    }
                    'v' => {
                        let (volume, new_input) = VolumeScanner.scan(rest)?;
                        rest = new_input;
                        Ok((MetaControl::ChangeVolume(volume), rest))
                    }
                    _ => {
                        Err(ScanError::Generic(format!(
                            "Expected MetaControl: i= or v=, found {}=",
                            first
                        )))
                    }
                }
            } else {
                Err(ScanError::Generic(format!("Expected '=' to follow meta control character {first}")))
            }
        } else {
            Err(ScanError::Generic("Expected MetaControl".to_string()))
        }
    }
}

impl Scanner for NonTerminalScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // scan [-a-zA-Z] and return largest prefix
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first.is_alphabetic() || first == '-' {
                let mut non_terminal = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '-' {
                        non_terminal.push(c);
                    } else {
                        return Ok((non_terminal, chars.as_str()));
                    }
                }
                Ok((non_terminal, chars.as_str()))
            } else {
                Err(ScanError::Generic(format!("Expected NonTerminal but got {first}")))
            }
        } else {
            Err(ScanError::Generic(format!("Expected NonTerminal, but it's an empty string")))
        }
    }
}

impl Scanner for InstrumentScanner {
    type Output = Instrument;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // scan instrument name
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first.is_alphabetic() {
                let mut instrument = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_alphanumeric() || c == '_' {
                        instrument.push(c);
                    } else {
                        return Ok((instrument.parse().unwrap(), chars.as_str()));
                    }
                }
                Ok((instrument.parse().unwrap(), chars.as_str()))
            } else {
                Err(ScanError::Generic("Expected Instrument".to_string()))
            }
        } else {
            Err(ScanError::Generic("Expected Instrument".to_string()))
        }
    }
}

impl Scanner for VolumeScanner {
    type Output = Volume;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        // scan volume value
        let mut chars = input.chars();
        if let Some(first) = chars.next() {
            if first.is_digit(10) {
                let mut volume = first.to_string();
                while let Some(c) = chars.next() {
                    if c.is_digit(10) {
                        volume.push(c);
                    } else {
                        return Ok((Volume(volume.parse().unwrap()), chars.as_str()));
                    }
                }
                Ok((Volume(volume.parse().unwrap()), chars.as_str()))
            } else {
                Err(ScanError::Generic("Expected Volume".to_string()))
            }
        } else {
            Err(ScanError::Generic("Expected Volume".to_string()))
        }
    }
}

/// Assume that exactly 1 opening char has already been found. Find the next closing char.
fn find_matching(input: &str, open: char, close: char) -> Option<usize> {
    let mut stack = 1;
    for (i, c) in input.chars().enumerate() {
        if c == open {
            stack += 1;
        } else if c == close {
            stack -= 1;
            if stack == 0 {
                return Some(i);
            }
        }
    }
    None
}

pub struct StringScanner(String);

impl Scanner for StringScanner {
    type Output = String;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        if input.starts_with(&self.0) {
            Ok((self.0.clone(), &input[self.0.len()..]))
        } else {
            Err(ScanError::Generic(format!("Expected string: {}", self.0)))
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

pub struct ConsumeScanner<S>(S);

pub struct MapInputScanner<S, F> {
    scanner: S,
    mapper: F,
}

pub fn trim<S>(scan: S) -> impl Scanner<Output=S::Output>
where
    S: Scanner,
{
    scan_map_input(scan, |s| s.trim_start().trim_end())
}

pub fn consume<S>(scan: S) -> impl Scanner<Output=S::Output>
where
    S: Scanner,
{
    ConsumeScanner(scan)
}

pub fn scan_map<S, F, T>(scan: S, map: F) -> impl Scanner<Output=T>
where
    S: Scanner,
    F: Fn(S::Output) -> T,
{
    MapScanner {
        scanner: scan,
        mapper: map,
    }
}

pub fn scan_map_input<S, F>(scan: S, map: F) -> impl Scanner<Output=S::Output>
where
    S: Scanner,
    F: Fn(&str) -> &str,
{
    MapInputScanner {
        scanner: scan,
        mapper: map,
    }
}

pub fn kleene<S>(scan: S) -> impl Scanner<Output=Vec<S::Output>>
where
    S: Scanner,
{
    KleeneScan(scan)
}

pub fn concat<S, T, U, V>(scan1: S, scan2: T) -> impl Scanner<Output=(U, V)>
where
    S: Scanner<Output=U>,
    T: Scanner<Output=V>,
{
    ConcatScan(scan1, scan2)
}

pub fn disjoint<S, T, U>(
    prefix1: ScanPrefix,
    scan1: S,
    prefix2: Option<ScanPrefix>,
    scan2: T,
) -> impl Scanner<Output=U>
where
    S: Scanner<Output=U>,
    T: Scanner<Output=U>,
{
    DisjointScan {
        scanner_a: (prefix1, scan1),
        scanner_b: (prefix2, scan2),
    }
}

impl<S, T, U, V> Scanner for ConcatScan<S, T>
where
    S: Scanner<Output=U>,
    T: Scanner<Output=V>,
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
    S: Scanner<Output=U>,
    T: Scanner<Output=U>,
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

impl<S> Scanner for KleeneScan<S>
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

impl<S, T, U> Scanner for MapScanner<S, T>
where
    S: Scanner,
    T: Fn(S::Output) -> U,
{
    type Output = U;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        self.scanner
            .scan(input)
            .map(|(output, new_input)| ((self.mapper)(output), new_input))
    }
}

impl<S> Scanner for ConsumeScanner<S>
where
    S: Scanner,
{
    type Output = S::Output;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        self.0.scan(input).and_then(|(output, new_input)| {
            if new_input.is_empty() {
                Ok((output, new_input))
            } else {
                Err(ScanError::Generic("Did not consume entire input".to_string()))
            }
        })
    }
}

impl<S, T> Scanner for MapInputScanner<S, T>
where
    S: Scanner,
    T: Fn(&str) -> &str,
{
    type Output = S::Output;

    fn scan<'a>(&self, input: &'a str) -> Result<(Self::Output, &'a str)> {
        self.scanner
            .scan((self.mapper)(input))
            .map(|(output, new_input)| (output, new_input))
    }
}


#[cfg(test)]
mod test {
    use crate::cfg::scan::{consume, ConsumeScanner, DurationScanner, GrammarScanner, InstrumentScanner, MetaControlScanner, MusicPrimitiveRepeatScanner, MusicPrimitiveScanner, MusicStringScanner, NonTerminalScanner, NoteScanner, ProductionScanner, Scanner, SymbolScanner, TerminalScanner, VolumeScanner};

    #[test]
    fn test_1() {
        let input = "start S\nS = [3][:4c<1> :4d :_ :f# :g :c ::i=sine B]\nB = :0c";
        let scanner = consume(GrammarScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_instrument() {
        let input = "sine";
        let scanner = ConsumeScanner(InstrumentScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_duration() {
        let input = "<1/4>";
        let scanner = ConsumeScanner(DurationScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_volume() {
        let input = "20";
        let scanner = ConsumeScanner(VolumeScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_note() {
        let input = "4c#";
        let scanner = ConsumeScanner(NoteScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rest() {
        let input = "_";
        let scanner = ConsumeScanner(NoteScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_control() {
        let input = "i=sine";
        let scanner = ConsumeScanner(MetaControlScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_meta_control_terminal() {
        let input = ":i=sine";
        let scanner = ConsumeScanner(TerminalScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_terminal() {
        let input = "4c<1>";
        let scanner = ConsumeScanner(TerminalScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonterminal() {
        let input = "S-b";
        let scanner = ConsumeScanner(NonTerminalScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn symbol_scanner_1() {
        let input = ":bb";
        let scanner = ConsumeScanner(SymbolScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn symbol_scanner_2() {
        let input = "::i=sine";
        let scanner = ConsumeScanner(SymbolScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn symbol_scanner_3() {
        let input = "T";
        let scanner = ConsumeScanner(SymbolScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn music_string_scanner_0() {
        // without any repeats or splits so far
        let input = ":4c<1> :4d :_ :f# :g :c ::i=sine Ba-c";
        let scanner = ConsumeScanner(MusicStringScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn music_string_scanner_1() {
        // without any repeats or splits so far
        let input = ":4c<1> :4d :_ :f# :g :c ::i=sine B";
        let scanner = ConsumeScanner(MusicStringScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn music_primitive_repeat_scanner() {
        let input = "[3][:4c<1> :4d :_ :f# :g :c ::i=sine B]";
        let scanner = ConsumeScanner(MusicPrimitiveRepeatScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn music_primitive_split_scanner() {
        let input = "{:4c<1> :4d :_ :f# :g :c ::i=sine B | :4c<1> :4d :_ :f# :g :c ::i=sine B }";
        let scanner = ConsumeScanner(MusicPrimitiveScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn music_string_scanner_2() {
        // with splits and repeats
        let input = "{:4c<1> :4d :_ :f# :g :c ::i=sine B | [3][:4c<1> :4d :_ :f# :g :c ::i=sine B]}";
        let scanner = ConsumeScanner(MusicStringScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }

    #[test]
    fn production_scanner_1() {
        let input = "S = [3][:4c<1> :4d :_ :f# :g :c ::i=sine B]";
        let scanner = ConsumeScanner(ProductionScanner);
        let result = scanner.scan(input);
        println!("result: {result:#?}");
        assert!(result.is_ok());
    }


}