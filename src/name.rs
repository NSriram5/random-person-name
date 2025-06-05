#[derive(Debug, Clone, Copy)]

/// A tagged enum with to flag if the name is left or right biased in terms of null padding
pub enum PaddingBias {
    /// The array of characters is padded with None characters to the farthest right
    Left,
    /// The array of characters is padded with None characters to teh farthest left
    Right
}

/// A stack allocated struct to hold information about the name being created.
#[derive(Debug, Clone, Copy)]
pub struct Name<const N: usize>{
    /// The text fo the name as an array of optional chars. A left pad biased name will put nones in later elements of the array. A right pad bias will put nones in early elements of the array.
    pub text: [Option<char>; N],
    /// Unopinionated gender identification labelling. Label choices are open to a user of the API.
    pub gender_identity: [Option<char>; 16],
    /// An optional and unopinonated culture labeling data feature
    pub major_culture_label: Option<[Option<char>; 16]>,
    /// An optional and unopinionated culture labelling data feature that applies a smaller subset of a larger culture
    pub minor_culture_label: Option<[Option<char>; 16]>,
    /// An optional and unopinionated label used to infer the a narrative sentiment about the name. E.g. if the name inspires fear, intrigue, comfort
    pub sentiment_label: Option<[Option<char>; 16]>,
    /// An optional and unopinionated label used to imply a family group, potentially the smallest culture labelling data feature
    pub family_label: Option<[Option<char>; 16]>
}

impl<const N: usize> Name<N> {
    /// Create a new name using string slices and optional string slices. 
    pub fn new(
        text: &str,
        gender_ident: &str,
        padding_bias: PaddingBias,
        major_culture_label: Option<&str>,
        minor_culture_label: Option<&str>,
        sentiment_label: Option<&str>,
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
            sentiment_label: sentiment_label.map(|s| str_to_char_arr(s)),
            family_label: family_label.map(|s| str_to_char_arr(s)),
        }
    }
    /// Uses an array slice of string slices to create a batch of names all belonging within one label grouping.
    pub fn new_from_batch(
        texts: &[&str],
        gender_ident: &str,
        padding_bias: PaddingBias,
        major_culture_label: Option<&str>,
        minor_culture_label: Option<&str>,
        sentiment_label: Option<&str>,
        family_label: Option<&str>,
    ) -> Vec<Self> {
        texts.into_iter().map(|&text| {
            Self::new(text, gender_ident, padding_bias, major_culture_label, minor_culture_label, sentiment_label, family_label)
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