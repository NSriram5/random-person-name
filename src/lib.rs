//! # Name_Gen
//! 
//! A library for reading names and making guesses at derived names using ngrams to guess the next character in the sequence
//! 
//! ## Intended Goal
//! A primary goal of this project is to have a smaller memory footprint than implementations that store lists of names with the library or rely on a lookup within a corpus. While also providing a mechanism
//! that produces some know
//! 
//! ## Recommended usage
//! 1. Invoke a mutable instance of a `NameExperiment` N=2 or N=3 are reasonable starting points.
//! 2. Utilize `Name` struct to handle raw &str of name text or perform manual conversion from `&str` to `&[Option<char>]`. Dedupe if desired.
//! 3. Iterate through names and run `NameExperiment::read_positive_sample` on each.
//! 4. Utilize `NameExperiment::build_random_name`. Apply external analysis to separate valid names from non names.
//! 5. Reinforce the weights within the `NameExperiment` by continuing to call `NameExperiment::read_positive_sample` and `NameExperiment::read_negative_sample` using valid and invalid names.
//! 
//! ## Implementation details explained 
//! This library exports a struct of `NameExperiments` and supports the analysis and extraction of probability distributions of character combinations.
//! To start, define a new NameExperiments with a generic const parameter N. N indicates how many characters to look backwards while analyzing a name
//! (Values of N less than 2 will result in a panic when `NameExperiments::new()` is called).
//! The `NameExperiments::read_positive_sample` function can be used to iterate through a list of names. This library assumes that a user will utilize the `text` field in the included `Name` struct,
//! but this can be bypassed by passing an array slice of `Option<char>` into `read_positive_sample`
//! 
//! > Note: The `read_positive_sample` function makes no attempt to de-duplicate text that has already be read. If the same name is read into a NameExperiments struct weights around that name's character
//! > sequences will become stronger. This might not be the intent; users of this library are advised to apply filtering or de-duplication earlier in their data pipeline.
//! 
//! Aside from gaining data about names, the `NameExperiments` struct can also read array slices of characters that are decidedly not names. The determination of what is or isn't a name is up to the
//! user of the API. But as a starting point, this can help to de-weight ngrams that would result in long sequences of vowels, consonants or simply letters that don't often follow one another.
//! Use `NameExperiments::read_negative_sample` to update weights that should correspond de-emphasized character sequences.
//! 
//! > Note: Again, `read_negative_sample` does not de-duplicate names.
//! 
//! ### Runtime Memory impact
//! Under the hood, the weights of the samples are stored within Four total `Vec` that are size allocated when "new" is called. Two of the `Vec` instances are used to hold observations about character
//! sequences and the count of an N+1 character observations in an array of length corresponding to the number of `ValidChar` variants.
//! The other two `Vec` instances hold observation data about character type sequences and following character type encounters in an array of length corresponding to the number of `CharType` variants.
//! 
//! All observation is stored in u8 format to minimize the memory impact of the weights (see Intended Goal), but analysis of larger data sets with frequent occurences of the same ngram sets may prove this
//! primitive too small.
//! Given an `N`` number of preceding characters assuming that there are 29 valid characters and 10 character types
//! the `NameExperiment` holds two `Vec` of capacity `29^N` and each array within the vec will be size 29 bytes. Meanwhile the two char_type sample weights will be `10^N` with arrays of size 10 bytes.
//! In the case of `N=2` memory footprint is estimated to be 51 kB. In the case of `N=3` memory footprint is estimated to be 1.4 MB.
//! > For reference: In a system that loads a corpus of names (of average length 8). 1.4 MB could hold around 22,400 names. But would be dependant on a user to provide the names.
//! 
//! ## TODO
//! * Exports weights and import weights to facilitate storage and retrieval between reinforcement sessions.
//! * Measure runtime memory impact and compare to estimated
//! * Estimates provided in the runtime memory impact imply that names could be generated with significantly lower memory consumption if the system relies on lower dimensions of character
//!  encoding (e.g. character type classifications) instead of using lengthier ngrams.
//! 
#![warn(missing_docs)]
use std::vec;
use fastrand::{f64 as rand_float};
use ngramweights::NGramWeights;


mod validchars;
mod char_types;
mod ngramweights;
mod name;
mod test_input_names;

pub use crate::name::{Name, PaddingBias};
pub use crate::validchars::{ValidChar};
pub use crate::char_types::{CharType};

#[derive(Debug,Copy,Clone)]
enum TestType {
    Pos,
    Neg
}

/// A datastructure that holds a variety of weights from reading lists of names and not-names. A NameExperiments struct is the primary way to read and generate derived names based on a body of text.
/// 
/// Within a name experiment are vectors used to store weighting information. `N` is of type `usize` and indicates the number of ngrams that will be studied. For example: if `N=2` then two characters
/// will be used as an experiment or the third.
/// 
/// The number of characters that are include in a character sequence experiment also correlates to the experiment around character types. Some character sound types require analysis of 3 characters to be effective
/// at correctly categorizing how a character influences phonetics in the word. E.g. 'Niche'
pub struct NameExperiments<const N: usize> {
    positive_char_samples: NGramWeights<N, {ValidChar::VARIANTCOUNT as usize}>,
    negative_char_samples: NGramWeights<N, {ValidChar::VARIANTCOUNT as usize}>,
    positive_char_type_samples: NGramWeights<N, {CharType::VARIANTCOUNT}>,
    negative_char_type_samples: NGramWeights<N, {CharType::VARIANTCOUNT}>,
    name_sizes: (Vec<usize>, usize)
}

impl<const N: usize> NameExperiments<N> {
    /// Create a new instance of a naming experiment. Ready to recieve names after created.
    pub fn new() -> Self {
        if N < 2 {
            panic!("N must be at least 2");
        }
        if (ValidChar::VARIANTCOUNT as usize).checked_pow(N as u32).is_none() {
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
    fn read_sample(&mut self, text: &[Option<char>], test_type: TestType) -> Result<(),String> {
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
    /// Reads a sample and applies it to the positive test case weights matrix
    pub fn read_positive_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Pos)
    }
    /// Reads a sample and applies it to the negative test case weights matrix
    pub fn read_negative_sample(&mut self, text: &[Option<char>]) -> Result<(),String> {
        self.read_sample(text, TestType::Neg)
    }
    /// Takes a character sequence, a character type sequence, a current count of characters in the word, applies optional positive and easing values and produces a probability distribution over the array of valid characters.
    /// 
    /// ## Parameters
    /// * char_sequence: an array slice of ValidChar to be analysed. Minimum length should be N. Where an experiment of an N character sequence would result in a N+1 character observation.
    /// * char_type_seq: an array slice of CharType to be analysed. Minimum length should be N. Where an experiment of an N character sequence would result in a N+1 character observation.
    /// * character_count: Provide context to the probability distribution of how far along within the name the next guess character would be. Assists with name termination probabilities.
    /// * pos_easing_scale, neg_easing_scale: Optional parameters to control how much easing is applied to the positive observation cases and how much is applied to the negative observation cases. Defaults to `1.0` if `None` is passed
    /// * square_probabilities: Optional parameter to control if a final square of probabilities is applied to "sharpen" the probability distribution. Can result in a bias to repeat names in the input list, But can assist in reducing the incidence of randomness on the output.
    /// 
    /// Use this function if the intent is to combine multiple probability distrubtions and handle letter guessing with other logic.
    /// Defer to using `guess_next_char` if the intent is to resolve to a single character.
    /// Defer to using `build_random_name` if the intent is to progress through a whole name generation loop.
    /// Easing defaults are `1.0` for positive test cases and `1.0` for negative test cases.
    /// This means that for any given character sequence resulting in `s` observations of a following character amongst a larger population of `n` observations the probability will be
    /// 
    /// `(s+1.0)/(n+count_chars)`
    /// 
    /// where `count_char` is the total number of character choices.
    /// 
    /// See: [Rule of Succession](https://en.wikipedia.org/wiki/Rule_of_succession)
    pub fn generate_probability_distribution(
        &self,
        char_seq: &[ValidChar],
        char_type_seq: &[CharType], 
        character_count: u8, 
        pos_easing_scale: Option<f64>,
        neg_easing_scale: Option<f64>,
        square_probabilities: Option<bool>
    ) -> Result<([f64; ValidChar::VARIANTCOUNT as usize], f64, [ValidChar;4]), String> {
        let pos_easing_scale = pos_easing_scale.unwrap_or(1.0);
        let neg_easing_scale = neg_easing_scale.unwrap_or(1.0);
        let mut char_4_sequence: [ValidChar; 4] = [ValidChar::null, ValidChar::null, ValidChar::null, ValidChar::null];
        for i in 0..3 {
            char_4_sequence[4-2-i] = *char_seq.get(char_seq.len()-1-i).unwrap_or(&ValidChar::null);
        }
        // Use existing details about the ngrams to produce a probability distribution of the chars without their types factored in.
        // Build a mapping to which predicted characters map to which character types
        let (pos_chars, pos_char_sum) = self.positive_char_samples.get_row_and_sum(char_seq)?;
        let (neg_chars, neg_char_sum) = self.negative_char_samples.get_row_and_sum(char_seq)?;
        let mut combined_char_probabilities: [f64; ValidChar::VARIANTCOUNT as usize] = [0.0; ValidChar::VARIANTCOUNT as usize];
        let mut char_type_mapping: [Vec<usize>; CharType::VARIANTCOUNT] = [const {vec![]}; CharType::VARIANTCOUNT];
        for i in 0..ValidChar::VARIANTCOUNT as usize {
            let inv_neg_chars_p = neg_char_sum - (neg_chars[i] as usize);
            // Applying easing to avoid NaNs while combineing negative and positive probabilities.
            combined_char_probabilities[i] = if neg_char_sum == 0 {
                (pos_chars[i] as f64 + pos_easing_scale) / (pos_char_sum as f64 + (pos_easing_scale * ValidChar::VARIANTCOUNT as f64))
            } else {
                ((pos_chars[i] as f64 + pos_easing_scale) / (pos_char_sum as f64 + (pos_easing_scale * ValidChar::VARIANTCOUNT as f64))) *
                    ((inv_neg_chars_p as f64 + pos_easing_scale)/ (neg_char_sum as f64 + (neg_easing_scale * ValidChar::VARIANTCOUNT as f64)))
            };
            char_4_sequence[3] = ValidChar::ALLCHARS[i];
            let mapped_char_type = CharType::try_from(&char_4_sequence)?;
            char_type_mapping[mapped_char_type as usize].push(i);
        }
        // Use existing details about ngrams of character types to build distribution of character types.
        // Apply existing character type mappings and their probabilities to the existing probabilities factored so far.
        let (pos_char_types, pos_char_type_sum) = self.positive_char_type_samples.get_row_and_sum(char_type_seq)?;
        let (neg_char_types, neg_char_type_sum) = self.negative_char_type_samples.get_row_and_sum(char_type_seq)?;
        for i in 0..CharType::VARIANTCOUNT {
            let inv_neg_char_type_p = neg_char_type_sum - (neg_char_types[i] as usize);
            // Applying easing to avoid NaNs while combineing negative and positive probabilities.
            let combined_type_p  = ((pos_char_types[i] as f64 + pos_easing_scale)/(pos_char_type_sum as f64 + (pos_easing_scale * CharType::VARIANTCOUNT as f64))) *
                ((inv_neg_char_type_p as f64 + neg_easing_scale)/(neg_char_type_sum as f64 + (neg_easing_scale * CharType::VARIANTCOUNT as f64)));
            for &j in char_type_mapping.get(i).unwrap() {
                combined_char_probabilities[j] *= combined_type_p;
            }
        }
        // Apply statistics about name endings to the probabilities
        {
            let probability_end_here: f64 = self.name_sizes.0[0..(character_count as usize)].iter().map(|&x| (x as f64)/self.name_sizes.1 as f64).sum();
            let probability_ends_in_future = 1.0 - probability_end_here;
            // println!("prob ends here: {probability_end_here}, prob ends in future: {probability_ends_in_future}");
            for i in 0..combined_char_probabilities.len()-1 {
                combined_char_probabilities[i] *= probability_ends_in_future / ValidChar::VARIANTCOUNT as f64;
            }
            combined_char_probabilities[combined_char_probabilities.len()-1] *= probability_end_here;
            // combined_char_probabilities[combined_char_probabilities.len()-1] = probability_end_here;
        }
        if square_probabilities.unwrap_or(true) {
            // Square the probabilities
            for i in 0..combined_char_probabilities.len() {
                combined_char_probabilities[i] *= combined_char_probabilities[i];
            }
        }

        let sum_of_probabilities = combined_char_probabilities.iter().sum::<f64>();
        if sum_of_probabilities.is_nan() {
            return Err(format!("Sum of probabilities produced a nan: {combined_char_probabilities:?}"));
        }
        Ok((combined_char_probabilities, sum_of_probabilities, char_4_sequence))

    }
    /// Takes a character sequence, a character type sequence, the current count of characters in a word, and guesses next character, its corresponding character type. If an error is encountered it produces a String based Err.
    /// 
    /// ## Parameters
    /// * char_seq: an array slice of ValidChar to be analysed. Minimum length should be N. Where an experiment of an N character sequence would result in a N+1 character observation.
    /// * char_type_seq: an array slice of CharType to be analysed. Minimum length should be N. Where an experiment of an N character sequence would result in a N+1 character observation.
    /// * current_character_count: Provide context to the probability distribution of how far along within the name the next guess character would be. Assists with name termination probabilities.
    /// 
    ///  
    pub fn guess_next_char(&self, char_seq: &[ValidChar], char_type_seq: &[CharType], current_char_count: u8) -> Result<(ValidChar, CharType), String> {
        let (char_probabilities, sum_of_probabilities, mut char_4_sequence) = self.generate_probability_distribution(
            char_seq, char_type_seq, 
            current_char_count, 
            None, 
            None,
            None
        )?;
        // println!("p: {char_probabilities:?}, p_sum: {sum_of_probabilities}, 4char_sequence: {char_4_sequence:?}");
        // println!("");
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
    /// Using the existing positive and negative weights the system will repetitively guess names until it encounteres a null character. Once the loop guesses a null character the function returns a resulting name in all lowercase letters as a String. If the function encounters an error it will produce a string based Err.
    /// 
    /// ## Parameters
    /// * hard_stop: An optional parameter to apply a strict control the number of characters produced. Defaults to `16` if `None` is provided
    pub fn build_random_name(&self, hard_stop: Option<u8>) -> Result<String,String> {
        let mut char_type_array: [CharType; N] = [CharType::Null;N];
        let mut char_array: [ValidChar; N] = [ValidChar::null;N];
        let mut name_string = String::new();
        let (mut next_char, mut next_char_type) = self.guess_next_char(&char_array, &char_type_array, name_string.len() as u8)?;
        while next_char != ValidChar::null && name_string.len() != hard_stop.unwrap_or(16) as usize {
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
    use crate::{name::{self, Name}, test_input_names::{INPUT_EUROPEAN_MALE_NAMES, INPUT_GOBLIN_NAMES, INPUT_GREEK_FEMALE_NAMES, INPUT_ORC_NAMES, NOT_NAMES}, NameExperiments};

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
        let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
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
        let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
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
            let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
            random_names.push(new_name);
        }
        print!("[");
        random_names.iter().for_each(|n| print!("\"{n}\", "));
        print!("]");
    }

        #[test]
    fn it_makes_a_random_greek_female_name() {
        let names: Vec<Name<16>> = Name::new_from_batch(
            INPUT_GREEK_FEMALE_NAMES,
            "female",
            name::PaddingBias::Left,
            Some("Greek"), None, None, None
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
            let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
            random_names.push(new_name);
        }
        print!("[");
        random_names.iter().for_each(|n| print!("\"{n}\", "));
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
            let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
            random_names.push(new_name);
        }
        print!("[");
        random_names.iter().for_each(|n| print!("\"{n}\", "));
        print!("]");
    }
}
