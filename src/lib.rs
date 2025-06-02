use std::{ops::Add, path::Iter};
use char_types::{CharType, CHAR_TYPE_COUNT};
use fastrand::{f64 as rand_float};
use ngramweights::NGramWeights;
use validchars::{ValidChar, VALID_CHAR_COUNT};

mod validchars;
mod char_types;
mod ngramweights;
mod name;



#[derive(Debug,Copy,Clone)]
pub enum TestType {
    Pos,
    Neg
}

pub struct NameExperiments<const N: usize, const M: usize> {
    positive_char_samples: NGramWeights<N, M, VALID_CHAR_COUNT>,
    negative_char_samples: NGramWeights<N, M, VALID_CHAR_COUNT>,
    positive_char_type_samples: NGramWeights<N, M, CHAR_TYPE_COUNT>,
    negative_char_type_samples: NGramWeights<N, M, CHAR_TYPE_COUNT>,
    finalized: bool
}

impl<const N: usize, const M: usize> NameExperiments<N, M> {
    pub fn new() -> Self {
        if N < 2 {
            panic!("N must be at least 2");
        }
        if (VALID_CHAR_COUNT as usize).checked_pow(N as u32).is_none() {
            panic!("Number of {} ngrams picked will result in overflow",N);
        }
        if (VALID_CHAR_COUNT as usize).pow(N as u32) != M {
            panic!("M must be {}^N. Instead, M: {} and N: {}", VALID_CHAR_COUNT, M, N);
        }
        Self { 
            positive_char_samples: NGramWeights::new(),
            negative_char_samples: NGramWeights::new(),
            positive_char_type_samples: NGramWeights::new(),
            negative_char_type_samples: NGramWeights::new(),
            finalized: false,
        }
    }
    pub fn read_sample(&mut self, text: &[Option<char>], test_type: TestType) -> Result<(),String> {
        let mut i = 0;
        let mut valid_chars: Vec<ValidChar> = Vec::with_capacity(text.len());
        let mut char_weights = match test_type {
            TestType::Pos => self.positive_char_samples,
            TestType::Neg => self.negative_char_samples,
        };
        let mut char_type_weights = match test_type {
            TestType::Pos => self.positive_char_type_samples,
            TestType::Neg => self.negative_char_type_samples,
        };
        // add ngrams of characters from sample to weights
        let mut n_gram = [ValidChar::null; N];
        while let Some(p_char) = text[i] {
            let p_char = &ValidChar::try_from(&p_char).unwrap_or(ValidChar::null);
            let _ = char_weights.add_to_weights(&n_gram,p_char);
            n_gram.rotate_left(1);
            n_gram[N-1] = *p_char;
            valid_chars.push(*p_char);
        }
        {
            // the last ngram should terminate the word. It needs to be added
            let p_char = ValidChar::null;
            let _ = char_weights.add_to_weights(&n_gram,&p_char);
        }
        // Make an array of character types using the previously derived valid chars
        let mut char_types: Vec<CharType> = Vec::with_capacity(text.len());
        for (i, p_char) in valid_chars.iter().enumerate() {
            let mut char_slice = [ValidChar::null; 4];
            for j in 0..char_slice.len() {
                if j>i {continue;}
                char_slice[i-j] = valid_chars[i-j];
            }
            let char_type = CharType::try_from(&char_slice)?;
            char_types.push(char_type);
        }
        // add ngrams of character types to their weights
        let mut char_type_slice = [CharType::Null; N];
        for i in 0..char_types.len() {
            let p_char= char_types[i];
            let _ = char_type_weights.add_to_weights(&char_type_slice, &p_char);
            char_type_slice.rotate_left(1);
            char_type_slice[N-1] = p_char;
        }
        {
            // add a final char type ngram to quantify word endings
            let p_char = CharType::Null;
            let _ = char_type_weights.add_to_weights(&char_type_slice, &p_char);

        }
        Ok(())
    }
    pub fn read_positive_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Pos)
    }
    pub fn read_negative_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Neg)
    }
    pub fn finalize(&mut self) -> Result<(), String> {
        let _ = self.positive_char_samples.finalize()?;
        let _ = self.negative_char_samples.finalize()?;
        let _ = self.positive_char_type_samples.finalize()?;
        let _ = self.negative_char_type_samples.finalize()?;
        self.finalized = true;
        Ok(())
    }
    pub fn guess_next_char(&self, char_seq: &[ValidChar], char_type_seq: &[CharType]) -> Result<(ValidChar, CharType), String> {
        let mut char_4_sequence: [ValidChar; 4] = [ValidChar::null, ValidChar::null, ValidChar::null, ValidChar::null];
        for i in 0..3 {
            char_4_sequence[4-2-i] = *char_seq.get(char_seq.len()-1-i).unwrap_or(&ValidChar::null);
        }
        let (pos_chars, pos_char_sum) = self.positive_char_samples.get_row_and_sum(char_seq)?;
        let (neg_chars, neg_char_sum) = self.negative_char_samples.get_row_and_sum(char_seq)?;
        let mut combined_char_probabilities: [f64; VALID_CHAR_COUNT] = [0.0; VALID_CHAR_COUNT];
        let mut char_type_mapping: [Vec<usize>; CHAR_TYPE_COUNT] = [const {vec![]}; CHAR_TYPE_COUNT];
        for i in 0..VALID_CHAR_COUNT {
            let inv_neg_chars_p = neg_char_sum - (neg_chars[i] as usize);
            combined_char_probabilities[i] = (pos_chars[i] as f64 / pos_char_sum as f64) * (inv_neg_chars_p as f64/ neg_char_sum as f64);
            char_4_sequence[3] = ValidChar::ALLCHARS[i];
            let mapped_char_type = CharType::try_from(&char_4_sequence)?;
            char_type_mapping[mapped_char_type as usize].push(i);
        }
        let (pos_char_types, pos_char_type_sum) = self.positive_char_type_samples.get_row_and_sum(char_seq)?;
        let (neg_char_types, neg_char_type_sum) = self.negative_char_type_samples.get_row_and_sum(char_seq)?;
        for i in 0..CHAR_TYPE_COUNT {
            let inv_neg_char_type_p = neg_char_type_sum - (neg_char_types[i] as usize);
            let combined_type_p  = (pos_char_types[i] as f64/pos_char_type_sum as f64)*(inv_neg_char_type_p as f64/neg_char_type_sum as f64);
            for &j in char_type_mapping.get(i).unwrap() {
                combined_char_probabilities[j] *= combined_type_p;
            }
        }
        let sum_of_probabilities = combined_char_probabilities.iter().sum::<f64>();
        let mut random_pick = rand_float() * sum_of_probabilities;
        let index_pick  = combined_char_probabilities.into_iter().enumerate().find_map(|(i, p)| {
            if p >= random_pick {return Some(i)} else {
                random_pick -= p;
                None
            }
        }).ok_or("Random pick failed to find a valid character within the range. This is unexpected.".to_string())?;
        char_4_sequence[3] = ValidChar::ALLCHARS[index_pick];
        let picked_char_type = CharType::try_from(&char_4_sequence)?;
        Ok((ValidChar::ALLCHARS[index_pick], picked_char_type))
    }
    pub fn build_random_name(&self, hard_stop: u8) -> Result<String,String> {
        let mut char_type_array: [CharType; N] = [CharType::Null;N];
        let mut char_array: [ValidChar; N] = [ValidChar::null;N];
        let mut name_string = String::new();
        let (mut next_char, mut next_char_type) = self.guess_next_char(&char_array, &char_type_array)?;
        let mut stop = hard_stop;
        while next_char != ValidChar::null && stop != 0 {
            name_string.push(char::from(next_char));
            char_array.rotate_left(1);
            char_array[N-1] = next_char;
            char_type_array.rotate_left(1);
            char_type_array[N-1] = next_char_type;
            (next_char, next_char_type) = self.guess_next_char(&char_array, &char_type_array)?;
            stop -= 1;
        }
        Ok(name_string)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_makes_a_random_name() {
        
    }
}
