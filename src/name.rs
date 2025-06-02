#[derive(Debug, Clone, Copy)]
pub enum PaddingBias {Left, Right}

#[derive(Debug, Clone, Copy)]
pub struct Name<const N: usize>{
    pub text: [Option<char>; N],
    pub gender_identity: [Option<char>; 16],
    pub major_culture_label: Option<[Option<char>; 16]>,
    pub minor_culture_label: Option<[Option<char>; 16]>,
    pub emotion_label: Option<[Option<char>; 16]>,
    pub family_label: Option<[Option<char>; 16]>
}

impl<const N: usize> Name<N> {
    pub fn new(
        text: &str,
        gender_ident: &str,
        padding_bias: PaddingBias,
        major_culture_label: Option<&str>,
        minor_culture_label: Option<&str>,
        emotion_label: Option<&str>,
        family_label: Option<&str>,
    ) -> Self {
        if text.len() > N-1 {panic!("Name too long")}
        if gender_ident.len() > 16 {panic!("Gender identity too long")}
        let mut chars = [None; N];
        text.chars().into_iter().enumerate().for_each(|(i, c)| {
            match padding_bias {
                PaddingBias::Left => {
                    if i<N {
                        chars[i] = Some(c.to_ascii_lowercase());
                    }
                },
                PaddingBias::Right => {
                    if i<N {
                        chars[N-i-1] = Some(c.to_ascii_lowercase());
                    }
                }
            }
        });
        match padding_bias{
            PaddingBias::Left => chars[text.len()] = Some('_'),
            PaddingBias::Right => chars[N-text.len()-1] = Some('_'),
        }
        let mut gen_chars = [None; 16];
        gender_ident.chars().into_iter().enumerate().for_each(|(i, c)| {
            if i<16 {
                gen_chars[i] = Some(c);
            }
        });
        Self {
            text: str_to_char_arr(text),
            gender_identity: str_to_char_arr(gender_ident),
            major_culture_label: major_culture_label.map(|s| str_to_char_arr(s)),
            minor_culture_label: minor_culture_label.map(|s| str_to_char_arr(s)),
            emotion_label: emotion_label.map(|s| str_to_char_arr(s)),
            family_label: family_label.map(|s| str_to_char_arr(s)),
        }
    }
    pub fn new_from_batch(
        texts: &[&str],
        gender_ident: &str,
        padding_bias: PaddingBias,
        major_culture_label: Option<&str>,
        minor_culture_label: Option<&str>,
        emotion_label: Option<&str>,
        family_label: Option<&str>,
    ) -> Vec<Self> {
        texts.into_iter().map(|&text| {
            Self::new(text, gender_ident, padding_bias, major_culture_label, minor_culture_label, emotion_label, family_label)
        }).collect()
    }
}


fn str_to_char_arr<const N: usize>(text:&str) -> [Option<char>; N] {
    let mut chars = [None; N];
    text.chars().into_iter().enumerate().for_each(|(i, c)| {
        if i<N {
            chars[i] = Some(c);
        }
    });
    chars
}