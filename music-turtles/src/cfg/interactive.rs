
// an interactive CFG needs to keep track of which production rules and replacements it has made
// so that they can be reversed. An interactive CFG can be rendered into a MusicString.

use std::collections::HashMap;
use rocket::yansi::Paint;
use serde::{Deserialize, Serialize};
use crate::cfg::{Grammar, MusicPrimitive, MusicString, Production, Symbol};

pub struct InteractiveCFG {
    grammar: Grammar,
    root: TracedString
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TracedString {
    original: MusicString,
    productions: HashMap<usize, (Production, TracedString)>
}

impl InteractiveCFG {
    pub fn new(grammar: Grammar, music_string: MusicString) -> InteractiveCFG {
        InteractiveCFG {
            grammar,
            root: TracedString::new(music_string)
        }
    }
}

impl TracedString {
    pub fn new(music_string: MusicString) -> TracedString {
        TracedString {
            original: music_string,
            productions: HashMap::new()
        }
    }

    pub fn render(&self) -> MusicString {
        let mut v = vec![];
        for (i, mp) in self.original.0.iter().enumerate() {
            if let Some((_p, ts)) = self.productions.get(&i) {
                let ms = ts.render();
                ms.0.into_iter().for_each(|mp| v.push(mp));
            } else {
                v.push(mp.clone());
            }
        }
        MusicString(v)
    }
}