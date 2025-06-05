use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct NGramWeights<const N: usize, const V: usize> {
    pub weights: Vec<[u8;V]>,
    pub sum: Vec<usize>,
}

impl<const N: usize, const V: usize> NGramWeights<N, V>
{
    pub fn new() -> Self {
        if V.checked_pow(N as u32).is_none() {
            panic!("Number of {} ngrams picked will result in overflow",N);
        }
        let mut weights = Vec::with_capacity(V.pow(N as u32));
        for _i in 0..(V.pow(N as u32)) {weights.push([0u8;V]);}
        let mut sum = Vec::with_capacity(V.pow(N as u32));
        for _i in 0..(V.pow(N as u32)) {sum.push(0);}
        NGramWeights {
            weights: weights,
            sum: sum,
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
        let index = self.get_row_index(char_seq)?;
        Ok((self.weights.get_mut(index).unwrap(), self.sum.get_mut(index).unwrap()))
    }
    pub fn add_to_weights<T>(&mut self, sequence: &[T], following_char: &T) -> Result<(),String>
        where usize: From<T>,
        T: Clone + Copy + Debug
    {
        if sequence.len() < (N) {return Err("Not enough characters in input character sequence".to_string())}
        let (row, sum) = self.get_mut_row_and_sum(sequence).expect("Previous check should have gaurded against character input length errors");
        let column = usize::from(*following_char);
        row[column] = row[column].checked_add(1).ok_or("Weights max capacity reached")?;
        *sum = sum.checked_add(1).ok_or("Max ngram experiments reached")?;
        Ok(())
    }
    pub fn apply_easing(&mut self, numerator: u8, demoninator: u8) -> Result<(),String> {
        self.weights.iter_mut().enumerate().for_each(|(index, row)| {
            let mut fraction = 1u8;
            while (self.sum.get(index).unwrap()/fraction as usize).checked_add((numerator as usize * V)/demoninator as usize).is_none() {fraction += 1;}
            while row.iter().any(|&w| u8::try_from(
                ((w as usize/fraction as usize) + numerator as usize)/(demoninator as usize)
            ).is_err()) {fraction += 1;}
            let sum = self.sum[index];
            self.sum.get_mut(index).replace(&mut ((sum / fraction as usize) + (f64::round((numerator as f64 / demoninator as f64) * V as f64)) as usize));
            for j in 0..row.len() {
                let w = row[j];
                row[j] = (row[j] / fraction) + (f64::round(numerator as f64 / demoninator as f64) as u8)
            }
        });
        Ok(())
    }
}
