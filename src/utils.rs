use std::collections::HashMap;
use std::hash::Hash;

pub fn dedup_vec<T: PartialEq>(vec: Vec<T>) -> Vec<T> {
    let mut output: Vec<T> = Vec::new();

    for el in vec.into_iter() {
        if !output.contains(&el) {
            output.push(el);
        }
    }

    output
}

pub fn x_vec<T: Clone>(f: fn(T, T) -> T, lhs: Vec<T>, rhs: Vec<T>) -> Vec<T> {
    if lhs.is_empty() {
        rhs
    } else if rhs.is_empty() {
        lhs
    } else {
        let mut out: Vec<T> = Vec::new();
        for l in lhs.iter() {
            for r in rhs.iter() {
                out.push(f(l.clone(), r.clone()));
            }
        }
        out
    }
}

pub fn cross_fold<K: Hash + Eq, V>(data: Vec<Vec<HashMap<K, V>>>) -> Vec<HashMap<K, V>> {
    data.into_iter()
        .map(|v| v.into_iter().fold(HashMap::new(), |acc, hm| acc.into_iter().chain(hm).collect()))
        .collect()
}

#[cfg(test)]
mod utils_tests {
    use crate::utils::x_vec;

    #[test]
    fn x_vec_simple() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];

        let result = x_vec(|lhs, rhs| lhs + rhs, v1, v2);

        assert_eq!(result, vec![5, 6, 7, 6, 7, 8, 7, 8, 9]);
    }
}