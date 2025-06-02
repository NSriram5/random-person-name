use crate::validchars::{ValidChar};
use std::fmt::Debug;

// M must be VALID_CHAR_COUNT^N
#[derive(Debug, Copy, Clone)]
pub struct NGramWeights<const N: usize, const M: usize, const V: usize> {
    weights: [[u8;V]; M],
    sum: [usize; M],
    finalized: bool,
}

impl<const N: usize, const M: usize, const V: usize> NGramWeights<N, M, V>
{
    pub fn new() -> Self {
        if V.checked_pow(N as u32).is_none() {
            panic!("Number of {} ngrams picked will result in overflow",N);
        }
        if V.pow(N as u32) != M {
            panic!("M must be {}^N. Instead, M: {} and N: {}", V, M, N);
        }
        NGramWeights {
            weights: [[0u8;V];M],
            sum: [0;M],
            finalized: false
        }
    }
    fn get_row_index<T>(&self, char_seq: &[T]) -> Result<usize,String>
        where usize: From<T>, T: Clone + Copy + Debug
    {
        if char_seq.len() < N {return Err("Not enough characters given to determine row".to_string())}
        let mut index = 0usize;
        for i in 0..N {
            let char = char_seq[i as usize];
            index += (V.pow(i as u32)) * (usize::from(char));
        }
        #[cfg(test)]
        {
            debug_assert!(index < self.weights.len(), "{index} is not less than {}. Reading from characters: {char_seq:?}, N is: {N}", self.weights.len());
        }
        Ok(index)
    }
    pub fn get_row<T>(&self, char_seq: &[T]) -> Result<[u8;V],String> 
        where usize: From<T>, T: Clone + Copy + Debug
    {
        let index = self.get_row_index(char_seq)?;
        Ok(self.weights[index])
    }
    pub fn get_row_and_sum<T>(&self, char_seq: &[T]) -> Result<([u8;V], usize),String> 
        where usize: From<T>, T: Clone + Copy + Debug
    {
        let index = self.get_row_index(char_seq)?;
        Ok((self.weights[index], self.sum[index]))
    }
    pub fn get_mut_row_and_sum<T>(&mut self, char_seq:&[T]) -> Result<(&mut [u8;V], &mut usize),String> 
        where usize: From<T>, T: Clone + Copy + Debug
    {
        if self.finalized {return Err("Cannot modify weights after finalization".to_string())}
        let index = self.get_row_index(char_seq)?;
        Ok((self.weights.get_mut(index).unwrap(), self.sum.get_mut(index).unwrap()))
    }
    // pub fn read_text<T>(&mut self, text: &[Option<char>]) -> Result<(),String>
    //     where usize: From<T>, T: Clone + Copy + Debug
    // {
    //     let mut i = 0;
    //     while let Some(p_char) = text[i] {
    //         let mut n_gram_vec = [ValidChar::null;N];
    //         for j in 0..N {
    //             if i<(N-j) {continue;}
    //             n_gram_vec[j] = text[i-(N-j)].map(|x| ValidChar::try_from(&x).unwrap_or(ValidChar::null))
    //                 .unwrap_or(ValidChar::null);
    //         }
    //         let _ = self.add_to_weights(&n_gram_vec, &ValidChar::try_from(&p_char).unwrap_or(ValidChar::null))?;
    //         i += 1;
    //     }
    //     let mut n_gram_vec = [ValidChar::null;N];
    //     for j in 0..N {
    //         n_gram_vec[j] = text[i-(N-j)].map(|x| ValidChar::try_from(&x).unwrap_or(ValidChar::null))
    //            .unwrap_or(ValidChar::null);
    //     }
    //     let _ = self.add_to_weights(&n_gram_vec, &ValidChar::null)?;        
    //     Ok(())
    // }
    pub fn add_to_weights<T>(&mut self, sequence: &[T], following_char: &T) -> Result<(),String>
        where usize: From<T>,
        T: Clone + Copy + Debug
    {
        if self.finalized {return Err("Cannot add to weights after finalization".to_string())}
        if sequence.len() < (N) {return Err("Not enough characters in input character sequence".to_string())}
        let (row, sum) = self.get_mut_row_and_sum(sequence).expect("Previous check should have gaurded against character input length errors");
        let column = usize::from(*following_char);
        row[column] = row[column].checked_add(1).ok_or("Weights max capacity reached")?;
        *sum = sum.checked_add(1).ok_or("Max ngram experiments reached")?;
        Ok(())
    }
    pub fn finalize(&mut self) -> Result<(),String> {
        for i in 0..M {
            let mut divisor = 1u8;
            for j in 0..V {
                while u8::try_from(
                    ((self.weights[i][j]+1)) / divisor
                ).is_err() {
                    divisor += 1;
                }
            }
            self.sum[i] = self.sum[i] + V;
        }
        self.finalized = true;
        Ok(())
    }
    // pub fn guess_next_char(&self, char_seq: &[ValidChar]) -> Result<ValidChar,String>{
    //     let (distribution_row, total) = self.get_row_and_sum(char_seq)?;
    //     let mut random_pick = rand_u64(0..total);
    //     // #[cfg(test)]
    //     // {
    //     //     println!("pick: {}, distriution row: {:?}, row_total:{}", random_pick, distribution_row, total);
    //     // }
    //     for i in 0..VALID_CHAR_COUNT {
    //         if random_pick <= distribution_row[i as usize] as u64 {
    //             let new_char = ValidChar::try_from(i as u8)?;
    //             return Ok(new_char);
    //         } else {
    //             random_pick -= distribution_row[i as usize] as u64;
    //         }
    //     }
    //     Err(format!("Could not find a valid char when given char sequence {:?}. Random pick was: {:?}. Row was {:?}.", char_seq, random_pick, distribution_row))
    // }
    // pub fn build_random_name(&self) -> Result<String,String>{
    //     let mut pre = [ValidChar::null;N];
    //     let mut output: String = String::new();
    //     let mut next = self.guess_next_char(&pre)?;
    //     let max_name_length = 64;
    //     while next != ValidChar::null && output.len() <= max_name_length {
    //         output.push(char::from(next));
    //         for i in 0..N-1{
    //             pre[i] = pre[i+1];
    //         }
    //         pre[N-1] = next;
    //         next = self.guess_next_char(&pre)?;
    //     }
    //     Ok(output)
    // }
}
