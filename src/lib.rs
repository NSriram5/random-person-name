use std::{cmp::Reverse, collections::BinaryHeap, ops::Add, path::Iter, vec};
use char_types::{CharType, CHAR_TYPE_COUNT};
use fastrand::{f64 as rand_float};
use ngramweights::NGramWeights;
use validchars::{ValidChar, VALID_CHAR_COUNT};

mod validchars;
mod char_types;
mod ngramweights;
mod name;
mod test_input_names;

#[derive(Debug,Copy,Clone)]
pub enum TestType {
    Pos,
    Neg
}

pub struct NameExperiments<const N: usize> {
    positive_char_samples: NGramWeights<N, VALID_CHAR_COUNT>,
    negative_char_samples: NGramWeights<N, VALID_CHAR_COUNT>,
    positive_char_type_samples: NGramWeights<N, CHAR_TYPE_COUNT>,
    negative_char_type_samples: NGramWeights<N, CHAR_TYPE_COUNT>,
    name_sizes: (Vec<usize>, usize)
}

impl<const N: usize> NameExperiments<N> {
    pub fn new() -> Self {
        if N < 2 {
            panic!("N must be at least 2");
        }
        if (VALID_CHAR_COUNT as usize).checked_pow(N as u32).is_none() {
            panic!("Number of {} ngrams picked will result in overflow",N);
        }
        NameExperiments { 
            positive_char_samples: NGramWeights::new(),
            negative_char_samples: NGramWeights::new(),
            positive_char_type_samples: NGramWeights::new(),
            negative_char_type_samples: NGramWeights::new(),
            name_sizes: (vec![0], 0),
        }
    }
    fn add_to_sizes_distribution(&mut self, chars: &[ValidChar]) -> () {
        while chars.len() > self.name_sizes.0.len()-1 {
            self.name_sizes.0.push(0);
        }
        self.name_sizes.0[chars.len()] += 1;
        self.name_sizes.1 += 1;
    }
    pub fn read_sample(&mut self, text: &[Option<char>], test_type: TestType) -> Result<(),String> {
        let mut i = 0;
        let mut valid_chars: Vec<ValidChar> = Vec::with_capacity(text.len());
        let char_weights = match test_type {
            TestType::Pos => &mut self.positive_char_samples,
            TestType::Neg => &mut self.negative_char_samples,
        };
        let char_type_weights = match test_type {
            TestType::Pos => &mut self.positive_char_type_samples,
            TestType::Neg => &mut self.negative_char_type_samples,
        };
        // add ngrams of characters from sample to weights
        let mut n_gram = [ValidChar::null; N];
        while let Some(p_char) = text[i] {
            let p_char = &ValidChar::try_from(&p_char).unwrap_or(ValidChar::null);
            let _ = char_weights.add_to_weights(&n_gram,p_char);
            n_gram.rotate_left(1);
            n_gram[N-1] = *p_char;
            valid_chars.push(*p_char);
            i += 1;
        }
        {
            // the last ngram should terminate the word. It needs to be added
            let p_char = ValidChar::null;
            let _ = char_weights.add_to_weights(&n_gram,&p_char);
        }
        // Make an array of character types using the previously derived valid chars
        let mut char_types: Vec<CharType> = Vec::with_capacity(text.len());
        for i in 0..valid_chars.len() {
            let mut char_slice = [ValidChar::null; 4];
            for j in 0..char_slice.len() {
                if (j+1)>i {continue;}
                char_slice[4-(j+1)] = valid_chars[i-(j+1)];
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
        self.add_to_sizes_distribution(&valid_chars);
        Ok(())
    }
    pub fn read_positive_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Pos)
    }
    pub fn read_negative_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Neg)
    }
    
    pub fn generate_probability_distribution(&self, char_seq: &[ValidChar], char_type_seq: &[CharType], character_count: u8, pos_easing_scale: Option<f64>, neg_easing_scale: Option<f64>) -> Result<([f64; VALID_CHAR_COUNT], f64, [ValidChar;4]), String> {
        let pos_easing_scale = pos_easing_scale.unwrap_or(0.1);
        let neg_easing_scale = neg_easing_scale.unwrap_or(0.1);
        let mut char_4_sequence: [ValidChar; 4] = [ValidChar::null, ValidChar::null, ValidChar::null, ValidChar::null];
        for i in 0..3 {
            char_4_sequence[4-2-i] = *char_seq.get(char_seq.len()-1-i).unwrap_or(&ValidChar::null);
        }
        // Use existing details about the ngrams to produce a probability distribution of the chars without their types factored in.
        // Build a mapping to which predicted characters map to which character types
        let (pos_chars, pos_char_sum) = self.positive_char_samples.get_row_and_sum(char_seq)?;
        let (neg_chars, neg_char_sum) = self.negative_char_samples.get_row_and_sum(char_seq)?;
        let mut combined_char_probabilities: [f64; VALID_CHAR_COUNT] = [0.0; VALID_CHAR_COUNT];
        let mut char_type_mapping: [Vec<usize>; CHAR_TYPE_COUNT] = [const {vec![]}; CHAR_TYPE_COUNT];
        for i in 0..VALID_CHAR_COUNT {
            let inv_neg_chars_p = neg_char_sum - (neg_chars[i] as usize);
            // Applying easing to avoid NaNs while combineing negative and positive probabilities.
            combined_char_probabilities[i] = if neg_char_sum == 0 {
                (pos_chars[i] as f64 + pos_easing_scale) / (pos_char_sum as f64 + (pos_easing_scale * VALID_CHAR_COUNT as f64))
            } else {
                ((pos_chars[i] as f64 + pos_easing_scale) / (pos_char_sum as f64 + (pos_easing_scale * VALID_CHAR_COUNT as f64))) *
                    ((inv_neg_chars_p as f64 + pos_easing_scale)/ (neg_char_sum as f64 + (neg_easing_scale * VALID_CHAR_COUNT as f64)))
            };
            char_4_sequence[3] = ValidChar::ALLCHARS[i];
            let mapped_char_type = CharType::try_from(&char_4_sequence)?;
            char_type_mapping[mapped_char_type as usize].push(i);
        }
        // Use existing details about ngrams of character types to build distribution of character types.
        // Apply existing character type mappings and their probabilities to the existing probabilities factored so far.
        let (pos_char_types, pos_char_type_sum) = self.positive_char_type_samples.get_row_and_sum(char_type_seq)?;
        let (neg_char_types, neg_char_type_sum) = self.negative_char_type_samples.get_row_and_sum(char_type_seq)?;
        for i in 0..CHAR_TYPE_COUNT {
            let inv_neg_char_type_p = neg_char_type_sum - (neg_char_types[i] as usize);
            // Applying easing to avoid NaNs while combineing negative and positive probabilities.
            let combined_type_p  = ((pos_char_types[i] as f64 + pos_easing_scale)/(pos_char_type_sum as f64 + (pos_easing_scale * CHAR_TYPE_COUNT as f64))) *
                ((inv_neg_char_type_p as f64 + neg_easing_scale)/(neg_char_type_sum as f64 + (neg_easing_scale * CHAR_TYPE_COUNT as f64)));
            for &j in char_type_mapping.get(i).unwrap() {
                combined_char_probabilities[j] *= combined_type_p;
            }
        }
        // Apply statistics about name endings to the probabilities
        {
            let probability_end_here: f64 = self.name_sizes.0[0..(character_count as usize)].iter().map(|&x| (x as f64)/self.name_sizes.1 as f64).sum();
            let probability_ends_in_future = 1.0 - probability_end_here;
            for i in 0..combined_char_probabilities.len()-1 {
                combined_char_probabilities[i] *= probability_ends_in_future;
            }
            combined_char_probabilities[combined_char_probabilities.len()-1] *= probability_end_here;
        }
        let sum_of_probabilities = combined_char_probabilities.iter().sum::<f64>();
        if sum_of_probabilities.is_nan() {
            return Err(format!("Sum of probabilities produced a nan: {combined_char_probabilities:?}"));
        }
        Ok((combined_char_probabilities, sum_of_probabilities, char_4_sequence))

    }
    pub fn guess_next_char(&self, char_seq: &[ValidChar], char_type_seq: &[CharType], current_char_count: u8) -> Result<(ValidChar, CharType), String> {
        let (char_probabilities, sum_of_probabilities, mut char_4_sequence) = self.generate_probability_distribution(char_seq, char_type_seq, current_char_count, None, None)?;
        let mut random_pick = rand_float() * sum_of_probabilities;
        let pick_start = random_pick;
        let index_pick  = char_probabilities.into_iter().enumerate().find_map(|(i, p)| {
            if p >= random_pick {return Some(i)} else {
                random_pick -= p;
                None
            }
        }).ok_or(format!("Random pick failed to pick a value. pick:{pick_start}, sum_of_probabilities: {sum_of_probabilities}"))?;
        char_4_sequence[3] = ValidChar::ALLCHARS[index_pick];
        let picked_char_type = CharType::try_from(&char_4_sequence)?;
        Ok((ValidChar::ALLCHARS[index_pick], picked_char_type))
    }
    pub fn build_random_name(&self, hard_stop: u8) -> Result<String,String> {
        let mut char_type_array: [CharType; N] = [CharType::Null;N];
        let mut char_array: [ValidChar; N] = [ValidChar::null;N];
        let mut name_string = String::new();
        let (mut next_char, mut next_char_type) = self.guess_next_char(&char_array, &char_type_array, name_string.len() as u8)?;
        while next_char != ValidChar::null && name_string.len() != 16 {
            name_string.push(char::from(next_char));
            char_array.rotate_left(1);
            char_array[N-1] = next_char;
            char_type_array.rotate_left(1);
            char_type_array[N-1] = next_char_type;
            (next_char, next_char_type) = self.guess_next_char(&char_array, &char_type_array, name_string.len() as u8)?;
        }
        Ok(name_string)
    }
}

#[cfg(test)]
mod tests {
    use crate::{name::{self, Name}, test_input_names::{INPUT_EUROPEAN_MALE_NAMES, INPUT_GOBLIN_NAMES, INPUT_ORC_NAMES, NOT_NAMES}, NameExperiments};

    // use super::*;

    #[test]
    fn it_makes_a_random_orc_name() {
        let names: Vec<Name<16>> = Name::new_from_batch(
            INPUT_ORC_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Orc"), None, None, None
        );
        let mut name_guess_experiments: NameExperiments<3> = NameExperiments::new();
        for n in names.iter() {
            let _ = name_guess_experiments.read_positive_sample(&n.text).unwrap();
        }
        let new_name = name_guess_experiments.build_random_name(16).unwrap();
        println!("Hello, {}!", new_name);
    }

    #[test]
    fn it_makes_a_random_goblin_name() {
        let names: Vec<Name<16>> = Name::new_from_batch(
            INPUT_GOBLIN_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Goblin"), None, None, None
        );
        let mut name_guess_experiments: NameExperiments<3> = NameExperiments::new();
        for n in names.iter() {
            let _ = name_guess_experiments.read_positive_sample(&n.text).unwrap();
        }
        let new_name = name_guess_experiments.build_random_name(16).unwrap();
        println!("Hello, {}!", new_name);
    }

    #[test]
    fn it_makes_a_random_western_male_name() {
        let names: Vec<Name<16>> = Name::new_from_batch(
            INPUT_EUROPEAN_MALE_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("European"), None, None, None
        );
        let mut name_guess_experiments: NameExperiments<3> = NameExperiments::new();
        for n in names.iter() {
            let _ = name_guess_experiments.read_positive_sample(&n.text).unwrap();
        }
        let not_names: Vec<Name<18>> = Name::new_from_batch(
            NOT_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Not"), None, None, None
        );
        for nn in not_names.iter() {
            let _ = name_guess_experiments.read_negative_sample(&nn.text).unwrap();
        }
        let mut random_names: Vec<String> = Vec::with_capacity(50);
        for _ in 0..50 {
            let new_name = name_guess_experiments.build_random_name(16).unwrap();
            random_names.push(new_name);
        }
        let new_name = name_guess_experiments.build_random_name(16).unwrap();
        print!("[");
        random_names.iter().for_each(|n| print!("\"{n}\","));
        print!("]");
    }

    #[test]
    fn it_makes_a_random_generic_male_name() {
        let names1: Vec<Name<16>> = Name::new_from_batch(
            INPUT_ORC_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Orc"), None, None, None
        );
        let names2: Vec<Name<16>> = Name::new_from_batch(
            INPUT_GOBLIN_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Goblin"), None, None, None
        );
        let names3: Vec<Name<16>> = Name::new_from_batch(
            INPUT_EUROPEAN_MALE_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("European"), None, None, None
        );
        let not_names: Vec<Name<18>> = Name::new_from_batch(
            NOT_NAMES,
            "male",
            name::PaddingBias::Left,
            Some("Not"), None, None, None
        );
        let mut name_guess_experiments: NameExperiments<3> = NameExperiments::new();
        for n in names1.iter().chain(names2.iter()).chain(names3.iter()) {
            let _ = name_guess_experiments.read_positive_sample(&n.text).unwrap();
        }
        for nn in not_names.iter() {
            let _ = name_guess_experiments.read_negative_sample(&nn.text).unwrap();
        }
        let mut random_names: Vec<String> = Vec::with_capacity(50);
        for _ in 0..50 {
            let new_name = name_guess_experiments.build_random_name(16).unwrap();
            random_names.push(new_name);
        }
        print!("[");
        random_names.iter().for_each(|n| print!("\"{n}\","));
        print!("]");
    }
}
