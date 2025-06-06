use crate::{name::{self, Name}, NameExperiments};
mod test_input_names;
use test_input_names::{INPUT_EUROPEAN_MALE_NAMES, INPUT_GOBLIN_NAMES, INPUT_GREEK_FEMALE_NAMES, INPUT_ORC_NAMES, NOT_NAMES};

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
