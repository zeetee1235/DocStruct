/// Korean Hangul Jamo composition
/// Combines separated Hangul jamos (ㄱ, ㅏ, etc.) into complete syllables (가, 나, etc.)
use unicode_normalization::UnicodeNormalization;

const HANGUL_BASE: u32 = 0xAC00; // '가'
const JONGSEONG_COUNT: u32 = 28;
const JUNGSEONG_COUNT: u32 = 21;

// 초성 (Initial consonants) - Choseong
const CHOSEONG_MAP: &[(char, u32)] = &[
    ('ᄀ', 0),
    ('ᄁ', 1),
    ('ᄂ', 2),
    ('ᄃ', 3),
    ('ᄄ', 4),
    ('ᄅ', 5),
    ('ᄆ', 6),
    ('ᄇ', 7),
    ('ᄈ', 8),
    ('ᄉ', 9),
    ('ᄊ', 10),
    ('ᄋ', 11),
    ('ᄌ', 12),
    ('ᄍ', 13),
    ('ᄎ', 14),
    ('ᄏ', 15),
    ('ᄐ', 16),
    ('ᄑ', 17),
    ('ᄒ', 18),
];

// 중성 (Vowels) - Jungseong
const JUNGSEONG_MAP: &[(char, u32)] = &[
    ('ᅡ', 0),
    ('ᅢ', 1),
    ('ᅣ', 2),
    ('ᅤ', 3),
    ('ᅥ', 4),
    ('ᅦ', 5),
    ('ᅧ', 6),
    ('ᅨ', 7),
    ('ᅩ', 8),
    ('ᅪ', 9),
    ('ᅫ', 10),
    ('ᅬ', 11),
    ('ᅭ', 12),
    ('ᅮ', 13),
    ('ᅯ', 14),
    ('ᅰ', 15),
    ('ᅱ', 16),
    ('ᅲ', 17),
    ('ᅳ', 18),
    ('ᅴ', 19),
    ('ᅵ', 20),
];

// 종성 (Final consonants) - Jongseong (0 = no final consonant)
const JONGSEONG_MAP: &[(char, u32)] = &[
    ('ᆨ', 1),
    ('ᆩ', 2),
    ('ᆪ', 3),
    ('ᆫ', 4),
    ('ᆬ', 5),
    ('ᆭ', 6),
    ('ᆮ', 7),
    ('ᆯ', 8),
    ('ᆰ', 9),
    ('ᆱ', 10),
    ('ᆲ', 11),
    ('ᆳ', 12),
    ('ᆴ', 13),
    ('ᆵ', 14),
    ('ᆶ', 15),
    ('ᆷ', 16),
    ('ᆸ', 17),
    ('ᆹ', 18),
    ('ᆺ', 19),
    ('ᆻ', 20),
    ('ᆼ', 21),
    ('ᆽ', 22),
    ('ᆾ', 23),
    ('ᆿ', 24),
    ('ᇀ', 25),
    ('ᇁ', 26),
    ('ᇂ', 27),
];

fn get_choseong_index(c: char) -> Option<u32> {
    CHOSEONG_MAP
        .iter()
        .find(|&&(ch, _)| ch == c)
        .map(|&(_, idx)| idx)
}

fn get_jungseong_index(c: char) -> Option<u32> {
    JUNGSEONG_MAP
        .iter()
        .find(|&&(ch, _)| ch == c)
        .map(|&(_, idx)| idx)
}

fn get_jongseong_index(c: char) -> Option<u32> {
    JONGSEONG_MAP
        .iter()
        .find(|&&(ch, _)| ch == c)
        .map(|&(_, idx)| idx)
}

fn is_choseong(c: char) -> bool {
    get_choseong_index(c).is_some()
}

fn is_jungseong(c: char) -> bool {
    get_jungseong_index(c).is_some()
}

fn is_jongseong(c: char) -> bool {
    get_jongseong_index(c).is_some()
}

fn is_hangul_jamo_or_compat(c: char) -> bool {
    matches!(
        c as u32,
        0x1100..=0x11FF | // Hangul Jamo
        0x3130..=0x318F | // Hangul Compatibility Jamo
        0xA960..=0xA97F | // Hangul Jamo Extended-A
        0xD7B0..=0xD7FF   // Hangul Jamo Extended-B
    )
}

fn remove_intra_jamo_whitespace(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut result = String::with_capacity(text.len());

    for (i, c) in chars.iter().enumerate() {
        if c.is_whitespace() {
            let prev = chars[..i].iter().rev().find(|ch| !ch.is_whitespace());
            let next = chars[i + 1..].iter().find(|ch| !ch.is_whitespace());
            let between_jamos = prev
                .zip(next)
                .map(|(p, n)| is_hangul_jamo_or_compat(*p) && is_hangul_jamo_or_compat(*n))
                .unwrap_or(false);
            if between_jamos {
                continue;
            }
        }
        result.push(*c);
    }

    result
}

/// Compose Hangul syllable from jamos
/// cho + jung = syllable without final consonant
/// cho + jung + jong = syllable with final consonant
fn compose_syllable(cho: u32, jung: u32, jong: u32) -> Option<char> {
    if cho >= 19 || jung >= 21 || jong >= 28 {
        return None;
    }
    let code =
        HANGUL_BASE + (cho * JUNGSEONG_COUNT * JONGSEONG_COUNT) + (jung * JONGSEONG_COUNT) + jong;
    char::from_u32(code)
}

/// Combine separated Hangul jamos into complete syllables
///
/// Example: "한글" -> "한글"
pub fn combine_hangul(text: &str) -> String {
    // 1) NFKC: compatibility jamo (ㄱㅏ) -> canonical jamo (가)
    // 2) remove only whitespace between jamos so ordinary word spacing survives
    // 3) NFC + custom composition pass for robustness
    let normalized = text.nfkc().collect::<String>();
    let compact = remove_intra_jamo_whitespace(&normalized);
    let nfc_text = compact.nfc().collect::<String>();

    let chars: Vec<char> = nfc_text.chars().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Try to compose a Hangul syllable
        if is_choseong(c) && i + 1 < chars.len() && is_jungseong(chars[i + 1]) {
            let cho_idx = get_choseong_index(c).unwrap();
            let jung_idx = get_jungseong_index(chars[i + 1]).unwrap();

            // Check for optional jongseong
            let (jong_idx, skip) = if i + 2 < chars.len() && is_jongseong(chars[i + 2]) {
                (get_jongseong_index(chars[i + 2]).unwrap(), 3)
            } else {
                (0, 2)
            };

            if let Some(syllable) = compose_syllable(cho_idx, jung_idx, jong_idx) {
                result.push(syllable);
                i += skip;
                continue;
            }
        }

        // Not a composable jamo sequence, keep as-is
        result.push(c);
        i += 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_simple() {
        // "한" = ᄒ + ᅡ + ᆫ
        assert_eq!(combine_hangul("한"), "한");

        // "글" = ᄀ + ᅳ + ᆯ
        assert_eq!(combine_hangul("글"), "글");

        // "한글" = 한 + 글
        assert_eq!(combine_hangul("한글"), "한글");
    }

    #[test]
    fn test_combine_mixed() {
        // Mixed with English
        assert_eq!(combine_hangul("Hello 한글"), "Hello 한글");

        // "이것은" = 이 + 거 + 스 + ᆫ (but last ᆫ alone won't compose)
        assert_eq!(combine_hangul("이것은"), "이것은");
    }

    #[test]
    fn test_no_jongseong() {
        // "가" = ᄀ + ᅡ (no jongseong)
        assert_eq!(combine_hangul("가"), "가");

        // "나다" = 나 + 다
        assert_eq!(combine_hangul("나다"), "나다");
    }

    #[test]
    fn test_compat_jamo_and_spacing() {
        assert_eq!(combine_hangul("ㄱㅏ"), "가");
        assert_eq!(combine_hangul("ᄒ ᅡ ᆫ ᄀ ᅳ ᆯ"), "한글");
        assert_eq!(combine_hangul("한 글 테스트"), "한 글 테스트");
    }
}
