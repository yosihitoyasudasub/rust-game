//! Cuts the embedded fonts down to the characters this game actually draws.
//!
//! Noto Sans JP ships every kanji, which was 4.5 MB of a 5.5 MB wasm binary - for a
//! browser game that is most of the download. Every Japanese character in the game
//! lives in a string literal in `src/main.rs`, and the rest of that file is ASCII, so
//! taking the whole file's character set gives a safe superset of what is needed.
//!
//! Re-run whenever new text is added:
//!     cargo run --release --manifest-path tools/subset-font/Cargo.toml

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Sources must be glyf-outline TTFs, not CFF/OTF: klippa subsets `glyf` but has no
/// CFF implementation, so an .otf comes back at ~94% of its original size.
const JOBS: &[(&str, &str)] = &[
    ("assets/ZenKakuGothicNew-Regular.ttf", "assets/jp-subset.ttf"),
    ("assets/JetBrainsMono-Regular.ttf", "assets/mono-subset.ttf"),
];

fn main() {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let text = std::fs::read_to_string(root.join("src/main.rs")).expect("read src/main.rs");

    let mut chars: BTreeSet<char> = text.chars().filter(|c| !c.is_control()).collect();
    // keep all printable ASCII even if some is unused today, so adding an English
    // string later cannot silently render as blanks
    chars.extend((0x20u8..=0x7e).map(|b| b as char));
    // punctuation that may arrive from formatting rather than from a literal
    chars.extend("　、。「」『』（）〜…・±×÷→←↑↓".chars());

    let unicodes: Vec<u32> = chars.iter().map(|c| *c as u32).collect();
    println!(
        "{} characters ({} non-ascii)",
        chars.len(),
        chars.iter().filter(|c| **c as u32 > 0x7f).count()
    );

    for (src, dst) in JOBS {
        let data = std::fs::read(root.join(src)).unwrap_or_else(|e| panic!("read {src}: {e}"));
        let out = fontcull::subset_font_data_unicode(&data, &unicodes, &[])
            .unwrap_or_else(|e| panic!("subset {src}: {e:?}"));
        std::fs::write(root.join(dst), &out).unwrap_or_else(|e| panic!("write {dst}: {e}"));
        println!(
            "{:>44} -> {:>7} bytes  ({:.1}% of {} )",
            dst,
            out.len(),
            out.len() as f64 / data.len() as f64 * 100.0,
            data.len()
        );
    }
}
