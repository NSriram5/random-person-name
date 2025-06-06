# Name_Gen

A library for reading names and making guesses at derived names using ngrams to guess the next character in the sequence

## Intended Goal
A primary goal of this project is to have a smaller memory footprint than implementations that store lists of names with the library or rely on a lookup within a corpus. While also providing a mechanism
that produces some know

## Recommended usage
1. Invoke a mutable instance of a `NameExperiment` N=2 or N=3 are reasonable starting points.
2. Utilize `Name` struct to handle raw &str of name text or perform manual conversion from `&str` to `&[Option<char>]`. Dedupe if desired.
3. Iterate through names and run `NameExperiment::read_positive_sample` on each.
4. Utilize `NameExperiment::build_random_name`. Apply external analysis to separate valid names from non names.
5. Reinforce the weights within the `NameExperiment` by continuing to call `NameExperiment::read_positive_sample` and `NameExperiment::read_negative_sample` using valid and invalid names.

## Examples
```
let mut name_guess_experiments: NameExperiments<3> = NameExperiments::new();
let orc_names: &[str] = &["Morgash", "Nargul", "Snarlgash"];
let names = Name::new_from_batch(orc_names,
    "male",
    PaddingBias::Left,
    Some("Orc"),
    None,
    None,
    None
);
for n in names.iter() {
    let _ = name_guess_experiments.read_positive_sample(&n.text).unwrap();
}
let new_name = name_guess_experiments.build_random_name(Some(16)).unwrap();
println!("Hello, {}!", new_name);
```

## Implementation details explained 
This library exports a struct of `NameExperiments` and supports the analysis and extraction of probability distributions of character combinations.
To start, define a new NameExperiments with a generic const parameter N. N indicates how many characters to look backwards while analyzing a name
(Values of N less than 2 will result in a panic when `NameExperiments::new()` is called).
The `NameExperiments::read_positive_sample` function can be used to iterate through a list of names. This library assumes that a user will utilize the `text` field in the included `Name` struct,
but this can be bypassed by passing an array slice of `Option<char>` into `read_positive_sample`

> Note: The `read_positive_sample` function makes no attempt to de-duplicate text that has already be read. If the same name is read into a NameExperiments struct weights around that name's character
> sequences will become stronger. This might not be the intent; users of this library are advised to apply filtering or de-duplication earlier in their data pipeline.

Aside from gaining data about names, the `NameExperiments` struct can also read array slices of characters that are decidedly not names. The determination of what is or isn't a name is up to the
user of the API. But as a starting point, this can help to de-weight ngrams that would result in long sequences of vowels, consonants or simply letters that don't often follow one another.
Use `NameExperiments::read_negative_sample` to update weights that should correspond de-emphasized character sequences.

> Note: Again, `read_negative_sample` does not de-duplicate names.

### Runtime Memory impact
Under the hood, the weights of the samples are stored within Four total `Vec` that are size allocated when "new" is called. Two of the `Vec` instances are used to hold observations about character
sequences and the count of an N+1 character observations in an array of length corresponding to the number of `ValidChar` variants.
The other two `Vec` instances hold observation data about character type sequences and following character type encounters in an array of length corresponding to the number of `CharType` variants.

All observation is stored in u8 format to minimize the memory impact of the weights (see Intended Goal), but analysis of larger data sets with frequent occurences of the same ngram sets may prove this
primitive too small.
Given an `N`` number of preceding characters assuming that there are 29 valid characters and 10 character types
the `NameExperiment` holds two `Vec` of capacity `29^N` and each array within the vec will be size 29 bytes. Meanwhile the two char_type sample weights will be `10^N` with arrays of size 10 bytes.
In the case of `N=2` memory footprint is estimated to be 51 kB. In the case of `N=3` memory footprint is estimated to be 1.4 MB.
> For reference: In a system that loads a corpus of names (of average length 8). 1.4 MB could hold around 22,400 names. But would be dependant on a user to provide the names.

## TODO
* Exports weights and import weights to facilitate storage and retrieval between reinforcement sessions.
* Measure runtime memory impact and compare to estimated
* Estimates provided in the runtime memory impact imply that names could be generated with significantly lower memory consumption if the system relies on lower dimensions of character
 encoding (e.g. character type classifications) instead of using lengthier ngrams.
