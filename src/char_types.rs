use crate::validchars::ValidChar;

/// A tagged enum to label characters as having kinds of phonetic sounds.
/// Analysis of ngrams of character sounds can potentially discover a rational reason why specific sounds may occur in a group of names.
/// The goal of this enum is to group categories so that their relationship to one another can be studied in a given identity group.
/// 
/// Excluded from the list of variants are 'FlipTap' and 'Trill' consonants. These would be more complex to deduce and would potentially
/// be more complicated to implement. 
/// 
/// The current implementation is naive and can likely be improved to consider where character sounds are formed (articulators).
/// 
/// (see: [Place of Articulation](https://en.wikipedia.org/wiki/Place_of_articulation))
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CharType {
    /// A vowel character that exists to produce its own sound
    VowelRoot=0,
    /// A character that exists in a word to produce a modification on a previous vowel
    VowelModifier=1,
    /// A character existing to break up a name, but that are more broadly considered punctuation. e.g. an apostrophe or a hyphen
    SemiPunctuation=2,
    /// A plosive consonant. Typically a 'P', 'B', 'T', 'K', 'D', 'G'
    Plosive=3, // P, B, T, K, D, G : Block flow of air when spoken then release of air
    /// A fricative consonant. Typically 'F', 'S', 'Z', 'Th'
    Fricative=4, //  F, S, Z, TH: Sound made by the friction of breath when air is pushed throuhg a narrow opening
    /// An affricate consonant. Typically 'Ch', 'J'
    Affricate=5, // Ch, J: Sounds that start with a plosive and end with fricative
    /// A nasal consonant. Typically 'M', 'N', 'NG'
    Nasal=6, // M, N, NG: Sounds that come the nose with air from the mouth being blocked
    /// A approximant consonant. Typically 'W', 'R',  'Y', 'L'
    Approximant=7, // W, R, Y, L: Sounds created by bringing tongue and lips or lips close but not enough to touch
    /// A silent character. Exists to cover the case where 'h' follows 'g' or 'c' follows 's'
    Silent=8, // No sound
    /// A null character. Corresponding to an empty character or a space
    Null=9
    // FlipTap=8, // T sometimes: Sounds created by contraction of muscles that causes tongue or lips to flick another
    // Trill=9, // A sound caused by vibrating lips/tongue against another.
}

impl CharType {
    /// A constant to quantify how many variations on character types there are.
    pub const VARIANTCOUNT: usize = 10;
}

impl TryFrom<&[ValidChar;4]> for CharType {
    type Error = String;

    fn try_from(value: &[ValidChar;4]) -> Result<Self, Self::Error> {
        if value.len() == 0 {return Err("No characters provided in sequence".to_string())}
        let mut val_iter = value.into_iter().rev();
        let _ = match val_iter.next().unwrap() {
            ValidChar::p | ValidChar::b | ValidChar::t | ValidChar::k | ValidChar::d | ValidChar::q => return Ok(Self::Plosive),
            ValidChar::f | ValidChar::s | ValidChar::v | ValidChar::x | ValidChar::z => return Ok(Self::Fricative),
            ValidChar::j => return Ok(Self::Affricate),
            ValidChar::w | ValidChar::r | ValidChar::l => return Ok(Self::Approximant),
            ValidChar::m | ValidChar::n => return Ok(Self::Nasal),
            ValidChar::apostrophe | ValidChar::dash => return Ok(Self::SemiPunctuation),
            ValidChar::null => return Ok(Self::Null),
            // cases where looking earlier in the word is necessary
            ValidChar::c => {
                if let Some(next_char) = val_iter.next() {
                    match next_char {
                        ValidChar::s => return Ok(Self::Silent),
                        _ => return Ok(Self::Plosive)
                    }
                } else {return Ok(Self::Plosive)}
            }
            ValidChar::h => {
                if let Some(next_char) = val_iter.next() {
                    match next_char {
                        ValidChar::c => return Ok(Self::Affricate),
                        ValidChar::t => return Ok(Self::Fricative),
                        ValidChar::g => return Ok(Self::Silent),
                        _ => return Ok(Self::Fricative)
                    }
                } else {return Ok(Self::Fricative)}
            },
            ValidChar::g => {
                if let Some(next_char) = val_iter.next() {
                    match next_char {
                        ValidChar::n => return Ok(Self::Nasal),
                        _ => return Ok(Self::Plosive)
                    }
                } else {return Ok(Self::Plosive)}
            },
            ValidChar::y => {
                if let Some(next_char) = val_iter.next() {
                    match next_char {
                        ValidChar::a | ValidChar::e | ValidChar::i | ValidChar::o | ValidChar::u => return Ok(Self::VowelModifier),
                        _ => return Ok(Self::VowelRoot)
                    }
                } else {return Ok(Self::Approximant)}
            },
            ValidChar::a | ValidChar::i | ValidChar::o | ValidChar::u => {
                if let Some(next_char) = val_iter.next() {
                    match next_char {
                        ValidChar::a | ValidChar::e | ValidChar::i | ValidChar::o | ValidChar::u => return Ok(Self::VowelModifier),
                        _ => return Ok(Self::VowelRoot)
                    }
                } else {return Ok(Self::VowelRoot)}
            },
            ValidChar::e => {
                if let Some(c1) = val_iter.next() {
                    match c1 {
                        ValidChar::a | ValidChar::e | ValidChar::i | ValidChar::o | ValidChar::u => return Ok(Self::VowelModifier),
                        _ => {
                            if let Some(c2) = val_iter.next() {
                                let c2 = match (c2, c1) {
                                    (ValidChar::t, ValidChar::h) | 
                                        (ValidChar::c, ValidChar::h) |
                                        (ValidChar::s, ValidChar::h) => {
                                            if let Some(ch) = val_iter.next() {ch} else {c2}
                                    },
                                    _ => c2
                                };
                                match c2 {
                                    ValidChar::a | ValidChar::e | ValidChar::i | ValidChar::o | ValidChar::u => return Ok(Self::VowelModifier),
                                    _ => return Ok(Self::VowelRoot)
                               }
                            } else {return Ok(Self::VowelRoot)}
                        }
                    }
                } else {return Ok(Self::VowelRoot)}
            }
        };
    }
}

impl From<CharType> for usize {
    fn from(value: CharType) -> Self {
        value as usize
    }
}