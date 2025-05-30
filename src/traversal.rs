pub fn generate_traversal_paths<const N: usize>() -> Vec<[usize; N]> {
    let mut paths = vec![];
    let mut start: [usize; N] = [0; N];
    for i in 0..N {
        start[i] = i;
    }
    paths.push(start);
    let mut stem:[Option<usize>; N] = [None::<usize>; N];
    let stem_size: usize = 0;
    permutation_recurser(start, &mut paths, &mut stem, stem_size, 0);
    paths
}

fn permutation_recurser<const N: usize>(
    nums: [usize; N],
    paths:&mut Vec<[usize; N]>,
    stem: &mut [Option<usize>; N],
    stem_size: usize,
    used: usize
) -> () {
    if stem_size == N {
        let mut new_path = [0; N];
        for (i, num) in stem.iter().enumerate() {
            new_path[i] = stem[i].expect("Stem should be filled. Unexpected condition");
        }
        paths.push(new_path);
        return;
    }
    for i in 0..nums.len() {
        if (used & (1 << i)) != 0 { continue; }
        stem[stem_size] = Some(nums[i]);
        permutation_recurser(nums, paths, stem, stem_size + 1, used | (1 << i));
        stem[stem_size] = None;
    }
}