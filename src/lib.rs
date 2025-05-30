use std::{ops::Add, path::Iter};
use fastrand::{u64 as rand_u64};

#[derive(Debug, Clone, Copy)]
pub enum PaddingBias {Left, Right}

#[derive(Debug, Clone, Copy)]
pub struct Name<const N: usize>{
    text: [Option<char>; N],
    gender_identity: [Option<char>; 16],
    major_culture_label: Option<[Option<char>; 16]>,
    minor_culture_label: Option<[Option<char>; 16]>,
    emotion_label: Option<[Option<char>; 16]>,
    family_label: Option<[Option<char>; 16]>
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

const VALID_CHAR_COUNT: u8 = 29;

#[derive(Debug,Clone,Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ValidChar {
    a=0,b=1,c=2,d=3,e=4,f=5,g=6,h=7,i=8,j=9,k=10,l=11,m=12,n=13,o=14,p=15,q=16,r=17,s=18,t=19,u=20,v=21,w=22,x=23,y=24,z=25,dash=26,apostrophe=27,null=28
}

impl TryFrom<&char> for ValidChar {
    type Error=String;
    fn try_from(c: &char) -> Result<Self, String> {
        let c_ident = c.to_ascii_lowercase() as u32 - 'a' as u32;
        match c_ident {
            0 => Ok(Self::a),
            1 => Ok(Self::b),
            2 => Ok(Self::c),
            3 => Ok(Self::d),
            4 => Ok(Self::e),
            5 => Ok(Self::f),
            6 => Ok(Self::g),
            7 => Ok(Self::h),
            8 => Ok(Self::i),
            9 => Ok(Self::j),
            10 => Ok(Self::k),
            11 => Ok(Self::l),
            12 => Ok(Self::m),
            13 => Ok(Self::n),
            14 => Ok(Self::o),
            15 => Ok(Self::p),
            16 => Ok(Self::q),
            17 => Ok(Self::r),
            18 => Ok(Self::s),
            19 => Ok(Self::t),
            20 => Ok(Self::u),
            21 => Ok(Self::v),
            22 => Ok(Self::w),
            23 => Ok(Self::x),
            24 => Ok(Self::y),
            25 => Ok(Self::z),
            _ => match c {
                '-'=> Ok(Self::dash),
                '\'' => Ok(Self::apostrophe),
                '\0' => Ok(Self::null),
                _ => Err(format!("{c} is an invalid character"))
            }
        }
    }
}

impl From<ValidChar> for char {
    fn from(value: ValidChar) -> Self {
        match value {
            ValidChar::apostrophe => '\'',
            ValidChar::dash => '-',
            ValidChar::null => '\0',
            _ => char::from_u32(value as u32 + 'a' as u32).unwrap()
        }
    }
}

impl TryFrom<u8> for ValidChar {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..26 => Ok(
                ValidChar::try_from(&char::from_u32('a' as u32 + value as u32).unwrap()).unwrap()
            ),
            26 => Ok(ValidChar::dash),
            27 => Ok(ValidChar::apostrophe),
            28 => Ok(ValidChar::null),
            _ => Err(format!("{value} is an invalid character"))
        }
    }
}

#[derive(Debug,Copy,Clone)]
pub enum TestType {
    Pos,
    Neg
}

// M must be VALID_CHAR_COUNT^N
#[derive(Debug, Copy, Clone)]
pub struct NGramWeights<const N: usize, const M: usize> {
    pos_weights: [[u8;VALID_CHAR_COUNT as usize]; M],
    neg_weights: [[u8;VALID_CHAR_COUNT as usize]; M],
    pos_sum: [u64; M],
    neg_sum: [u64; M],
    finalized: bool,
}

impl<const N: usize, const M: usize> NGramWeights<N, M> {
    pub fn new() -> Self {
        if (VALID_CHAR_COUNT as usize).checked_pow(N as u32).is_none() {
            panic!("Number of {} ngrams picked will result in overflow",N);
        }
        if (VALID_CHAR_COUNT as usize).pow(N as u32) != M {
            panic!("M must be {}^N. Instead, M: {} and N: {}", VALID_CHAR_COUNT, M, N);
        }
        NGramWeights { 
            pos_weights: [[0;VALID_CHAR_COUNT as usize];M],
            pos_sum: [0;M],
            neg_weights: [[0;VALID_CHAR_COUNT as usize];M],
            neg_sum: [0;M],
            finalized: false
        }
    }
    fn get_row_index(&self, char_seq: &[ValidChar]) -> Result<usize,String> {
        if char_seq.len() < N {return Err("Not enough characters given to determine row".to_string())}
        let mut index = 0usize;
        for i in 0..N {
            let char = char_seq[i as usize];
            index += ((VALID_CHAR_COUNT as usize).pow(i as u32)) * char as usize;
        }
        #[cfg(test)]
        {
            debug_assert!(index < self.pos_weights.len(), "{index} is not less than {}. Reading from characters: {char_seq:?}, N is: {N}", self.pos_weights.len());
        }
        Ok(index)
    }
    pub fn get_row(&self, char_seq: &[ValidChar],test_type: TestType) -> Result<[u8;VALID_CHAR_COUNT as usize],String> {
        let index = self.get_row_index(char_seq)?;
        match test_type {
            TestType::Pos => Ok(self.pos_weights[index]),
            TestType::Neg => Ok(self.neg_weights[index])
        }
    }
    pub fn get_row_and_sum(&self, char_seq: &[ValidChar], test_type: TestType) -> Result<([u8;VALID_CHAR_COUNT as usize], u64),String> {
        let index = self.get_row_index(char_seq)?;
        match test_type {
            TestType::Pos => Ok((self.pos_weights[index], self.pos_sum[index])),
            TestType::Neg => Ok((self.neg_weights[index], self.neg_sum[index])),
        }
    }
    pub fn get_mut_row_and_sum(&mut self, char_seq:&[ValidChar], test_type: TestType) -> Result<(&mut [u8;VALID_CHAR_COUNT as usize], &mut u64),String> {
        if self.finalized {return Err("Cannot modify weights after finalization".to_string())}
        let index = self.get_row_index(char_seq)?;
        match test_type {
            TestType::Pos => Ok((self.pos_weights.get_mut(index).unwrap(), self.pos_sum.get_mut(index).unwrap())),
            TestType::Neg => Ok((self.neg_weights.get_mut(index).unwrap(), self.neg_sum.get_mut(index).unwrap()))
        }
    }
    pub fn read_text(&mut self, text: &[Option<char>], test_type: TestType) -> Result<(),String> {
        let mut i = 0;
        while let Some(p_char) = text[i] {
            let mut n_gram_vec = [ValidChar::null;N];
            for j in 0..N {
                if i<(N-j) {continue;}
                n_gram_vec[j] = text[i-(N-j)].map(|x| ValidChar::try_from(&x).unwrap_or(ValidChar::null))
                    .unwrap_or(ValidChar::null);
            }
            let _ = self.add_to_weights(&n_gram_vec, &ValidChar::try_from(&p_char).unwrap_or(ValidChar::null))?;
            i += 1;
        }
        let mut n_gram_vec = [ValidChar::null;N];
        for j in 0..N {
            n_gram_vec[j] = text[i-(N-j)].map(|x| ValidChar::try_from(&x).unwrap_or(ValidChar::null))
               .unwrap_or(ValidChar::null);
        }
        let _ = self.add_to_weights(&n_gram_vec, &ValidChar::null)?;        
        Ok(())
    }
    fn add_to_weights(&mut self, char_seq: &[ValidChar], following_char: &ValidChar) -> Result<(),String> {
        if self.finalized {return Err("Cannot add to weights after finalization".to_string())}
        if char_seq.len() < (N) {return Err("Not enough characters in input character sequence".to_string())}
        let (row, sum) = self.get_mut_row_and_sum(char_seq).expect("Previous check should have gaurded against character input length errors");
        let column = *following_char as usize;
        row[column] = row[column].checked_add(1).ok_or("Weights max capacity reached")?;
        *sum = sum.checked_add(1).ok_or("Max ngram experiments reached")?;
        Ok(())
    }
    pub fn finalize(&mut self) -> Result<(),String> {
        for i in 0..M {
            let mut divisor = 1u64;
            for j in 0..VALID_CHAR_COUNT as usize {
                while u8::try_from(
                    ((self.pos_weights[i][j]+1) as u64)*(self.pos_sum[i]+2) / divisor
                ).is_err() {
                    divisor += 1;
                }
            }
            let mut new_sum = 0u64;
            for j in 0..VALID_CHAR_COUNT as usize {
                self.pos_weights[i][j] = u8::try_from(
                    ((self.pos_weights[i][j]+1) as u64)*(self.pos_sum[i]+2) / divisor
                ).expect("Divisor should have been scaled previously");
                new_sum += self.pos_weights[i][j] as u64;
            }
            self.pos_sum[i] = new_sum;
        }
        self.finalized = true;
        Ok(())
    }
    pub fn guess_next_char(&self, char_seq: &[ValidChar]) -> Result<ValidChar,String>{
        let (distribution_row, total) = self.get_row_and_sum(char_seq)?;
        let mut random_pick = rand_u64(0..total);
        #[cfg(test)]
        {
            println!("pick: {}, distriution row: {:?}, row_total:{}", random_pick, distribution_row, total);
        }
        for i in 0..VALID_CHAR_COUNT {
            if random_pick <= distribution_row[i as usize] as u64 {
                let new_char = ValidChar::try_from(i as u8)?;
                return Ok(new_char);
            } else {
                random_pick -= distribution_row[i as usize] as u64;
            }
        }
        Err(format!("Could not find a valid char when given char sequence {:?}. Random pick was: {:?}. Row was {:?}.", char_seq, random_pick, distribution_row))
    }
    pub fn build_random_name(&self) -> Result<String,String>{
        let mut pre = [ValidChar::null;N];
        let mut output: String = String::new();
        let mut next = self.guess_next_char(&pre)?;
        let max_name_length = 64;
        while next != ValidChar::null && output.len() <= max_name_length {
            output.push(char::from(next));
            for i in 0..N-1{
                pre[i] = pre[i+1];
            }
            pre[N-1] = next;
            next = self.guess_next_char(&pre)?;
        }
        Ok(output)
    }
}

const TWO_NGRAM_COUNT: usize = (VALID_CHAR_COUNT as usize) * (VALID_CHAR_COUNT as usize);
pub type TwoNgramWeights = NGramWeights<2, TWO_NGRAM_COUNT>;
const THREE_NGRAM_COUNT: usize = (VALID_CHAR_COUNT as usize) * (VALID_CHAR_COUNT as usize) * (VALID_CHAR_COUNT as usize);
pub type ThreeNgramWeights = NGramWeights<3, THREE_NGRAM_COUNT>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_makes_basic_datastructures_on_ingest() {
        let test_name: Name<16> = Name::new(
            "charles",
            "male",
            PaddingBias::Left,
            None,
            None,
            None,
            None
        );
        let mut ngram_weights: TwoNgramWeights= TwoNgramWeights::new();
        ngram_weights.read_text(&test_name.text).unwrap();
        let input_vec = [ValidChar::try_from(&'a').unwrap(),ValidChar::try_from(&'r').unwrap()];
        println!("row weights: {:?}",ngram_weights.get_row(&input_vec).unwrap());
    }

    #[test]
    fn it_makes_a_random_name() {
        let batch_of_names: [&str;54] = [
            "Adam", "Aiden", "Alexander", "Benjamin", "Caleb", "Christopher", "Daniel",
            "David", "Elijah", "Gabriel", "George", "Gregory", "Harold", "Henry", "Isaac",
            "James", "Jared", "Jason", "Jeremy", "Jesse", "Joel", "John", "Jonathan", "Joseph",
            "Joshua", "Justin", "Keith", "Kyle", "Lawrence", "Leonard", "Lucas", "Luther",
            "Malcolm", "Matthew", "Michael", "Nathaniel", "Nicholas", "Patrick", "Peter",
            "Philip", "Quinn", "Randall", "Richard", "Robert", "Roderick", "Ronald", "Russell",
            "Ryan", "Seth", "Stephen", "Terrance", "Theodore", "Thomas", "Timothy"
        ];
        let batch_of_names: Vec<Name<16>> = Name::new_from_batch(
            &batch_of_names,
            "male",
            PaddingBias::Left,
            None,
            None,
            None,
            None
        );
        let mut ngram_weights: TwoNgramWeights = TwoNgramWeights::new();
        batch_of_names.iter().for_each(|name| {
            let _ = ngram_weights.read_text(&name.text).unwrap();
        });
        let _ = ngram_weights.finalize().unwrap();
        let random_name = ngram_weights.build_random_name().unwrap();
        println!("Random name: {}", random_name);
    }
}
