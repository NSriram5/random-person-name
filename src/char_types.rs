use crate::validchars::ValidChar;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum CharType {
    VowelRoot=0,
    VowelModifier=1,
    SemiPunctuation=2,
    Plosive=3, // P, B, T, K, D, G : Block flow of air when spoken then release of air
    Fricative=4, //  F, S, Z, TH: Sound made by the friction of breath when air is pushed throuhg a narrow opening
    Affricate=5, // Ch, J: Sounds that start with a plosive and end with fricative
    Nasal=6, // M, N, NG: Sounds that come the nose with air from the mouth being blocked
    Approximant=7, // W, R, Y, L: Sounds created by bringing tongue and lips or lips close but not enough to touch
    Silent=8, // No sound
    Null=9
    // FlipTap=8, // T sometimes: Sounds created by contraction of muscles that causes tongue or lips to flick another
    // Trill=9, // A sound caused by vibrating lips/tongue against another.
}

pub const CHAR_TYPE_COUNT: usize = CharType::VARIANTCOUNT;

impl CharType {
    pub const VARIANTCOUNT: usize = 10;
}

impl TryFrom<&[ValidChar;4]> for CharType {
    type Error = String;

    fn try_from(value: &[ValidChar;4]) -> Result<Self, Self::Error> {
        if value.len() == 0 {return Err("No characters provided in sequence".to_string())}
        let mut val_iter = value.into_iter().rev();
        let _ = match val_iter.next().unwrap() {
            ValidChar::p | ValidChar::b | ValidChar::t | ValidChar::k | ValidChar::d | ValidChar::c | ValidChar::q => return Ok(Self::Plosive),
            ValidChar::f | ValidChar::s | ValidChar::v | ValidChar::x | ValidChar::z => return Ok(Self::Fricative),
            ValidChar::j => return Ok(Self::Affricate),
            ValidChar::w | ValidChar::r | ValidChar::l => return Ok(Self::Approximant),
            ValidChar::m | ValidChar::n => return Ok(Self::Nasal),
            ValidChar::apostrophe | ValidChar::dash => return Ok(Self::SemiPunctuation),
            ValidChar::null => return Ok(Self::Null),
            // cases where looking earlier in the word is necessary
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