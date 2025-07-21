use tree_sitter::Tree;
use treelint_core::{LintViolation, Rule};

pub struct InvisibleCharacters;

impl Rule for InvisibleCharacters {
    fn id(&self) -> &'static str {
        "RS025"
    }

    fn name(&self) -> &'static str {
        "invisible-characters"
    }

    fn description(&self) -> &'static str {
        "Detects invisible Unicode characters that may cause confusion"
    }

    fn explanation(&self) -> &'static str {
        "Invisible Unicode characters like zero-width spaces, RTL/LTR marks, and other \
         non-printable characters can cause subtle bugs and security issues. They're often \
         accidentally introduced when copying code from other sources."
    }

    fn check(&self, _tree: &Tree, source: &str) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        // Check the entire source for invisible characters
        let lines: Vec<&str> = source.lines().collect();
        for (line_idx, line) in lines.iter().enumerate() {
            for (char_idx, ch) in line.char_indices() {
                if is_invisible_character(ch) {
                    violations.push(LintViolation {
                        line: line_idx + 1,
                        column: char_idx + 1,
                        message: format!(
                            "Invisible character detected: {} (U+{:04X}). This may cause unexpected behavior",
                            get_character_name(ch),
                            ch as u32
                        ),
                        lint_name: self.name().to_string(),
                        lint_id: self.id().to_string(),
                    });
                }
            }
        }

        violations
    }
}

fn is_invisible_character(ch: char) -> bool {
    match ch {
        // Zero-width characters
        '\u{200B}' | // Zero Width Space
        '\u{200C}' | // Zero Width Non-Joiner
        '\u{200D}' | // Zero Width Joiner
        '\u{FEFF}' | // Zero Width No-Break Space (BOM)
        '\u{2060}' | // Word Joiner
        
        // Directional formatting characters
        '\u{202A}' | // Left-to-Right Embedding
        '\u{202B}' | // Right-to-Left Embedding
        '\u{202C}' | // Pop Directional Formatting
        '\u{202D}' | // Left-to-Right Override
        '\u{202E}' | // Right-to-Left Override
        '\u{2066}' | // Left-to-Right Isolate
        '\u{2067}' | // Right-to-Left Isolate
        '\u{2068}' | // First Strong Isolate
        '\u{2069}' | // Pop Directional Isolate
        
        // Other problematic invisible characters
        '\u{00AD}' | // Soft Hyphen
        '\u{034F}' | // Combining Grapheme Joiner
        '\u{061C}' | // Arabic Letter Mark
        '\u{115F}' | // Hangul Choseong Filler
        '\u{1160}' | // Hangul Jungseong Filler
        '\u{17B4}' | // Khmer Vowel Inherent AQ
        '\u{17B5}' | // Khmer Vowel Inherent AA
        '\u{180E}' | // Mongolian Vowel Separator
        '\u{3164}' | // Hangul Filler
        '\u{FFA0}' | // Halfwidth Hangul Filler
        
        // Variation selectors (can be invisible)
        '\u{FE00}'..='\u{FE0F}' | // Variation Selector-1 to 16
        '\u{E0100}'..='\u{E01EF}' // Variation Selector-17 to 256
        => true,
        
        _ => {
            // Check for other categories of invisible characters
            match unicode_general_category(ch) {
                // Format characters (except normal whitespace)
                UnicodeCategory::Format => !matches!(ch, ' ' | '\t' | '\n' | '\r'),
                // Other control characters
                UnicodeCategory::Control => !matches!(ch, '\t' | '\n' | '\r'),
                _ => false,
            }
        }
    }
}

#[derive(PartialEq)]
enum UnicodeCategory {
    Format,
    Control,
    Other,
}

fn unicode_general_category(ch: char) -> UnicodeCategory {
    // Simplified Unicode category detection
    let code = ch as u32;

    // Control characters (C0 and C1)
    if (0x00..=0x1F).contains(&code) || (0x7F..=0x9F).contains(&code) {
        return UnicodeCategory::Control;
    }

    // Format characters (simplified check for common ranges)
    if matches!(code,
        0x00AD | // Soft Hyphen
        0x061C | // Arabic Letter Mark
        0x200B..=0x200F | // Zero-width and directional chars
        0x202A..=0x202E | // Directional formatting
        0x2060..=0x2069 | // Word joiner and isolates
        0xFEFF | // BOM
        0xFFF9..=0xFFFB // Interlinear annotation chars
    ) {
        return UnicodeCategory::Format;
    }

    UnicodeCategory::Other
}

fn get_character_name(ch: char) -> &'static str {
    match ch {
        '\u{200B}' => "Zero Width Space",
        '\u{200C}' => "Zero Width Non-Joiner",
        '\u{200D}' => "Zero Width Joiner",
        '\u{FEFF}' => "Zero Width No-Break Space (BOM)",
        '\u{2060}' => "Word Joiner",
        '\u{202A}' => "Left-to-Right Embedding",
        '\u{202B}' => "Right-to-Left Embedding",
        '\u{202C}' => "Pop Directional Formatting",
        '\u{202D}' => "Left-to-Right Override",
        '\u{202E}' => "Right-to-Left Override",
        '\u{2066}' => "Left-to-Right Isolate",
        '\u{2067}' => "Right-to-Left Isolate",
        '\u{2068}' => "First Strong Isolate",
        '\u{2069}' => "Pop Directional Isolate",
        '\u{00AD}' => "Soft Hyphen",
        '\u{034F}' => "Combining Grapheme Joiner",
        '\u{061C}' => "Arabic Letter Mark",
        '\u{115F}' => "Hangul Choseong Filler",
        '\u{1160}' => "Hangul Jungseong Filler",
        '\u{17B4}' => "Khmer Vowel Inherent AQ",
        '\u{17B5}' => "Khmer Vowel Inherent AA",
        '\u{180E}' => "Mongolian Vowel Separator",
        '\u{3164}' => "Hangul Filler",
        '\u{FFA0}' => "Halfwidth Hangul Filler",
        '\u{FE00}'..='\u{FE0F}' => "Variation Selector",
        '\u{E0100}'..='\u{E01EF}' => "Variation Selector",
        _ => "Unknown Invisible Character",
    }
}
