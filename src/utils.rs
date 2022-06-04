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

pub fn cross_fold<K: Hash + Eq + Clone, V: Clone>(data: Vec<Vec<HashMap<K, V>>>) -> Vec<HashMap<K, V>> {
    data.into_iter().fold(vec![], |acc, hm| {
        x_vec(|lhs, rhs| lhs.into_iter().chain(rhs).collect(), acc, hm)
    })
}

#[cfg(test)]
mod utils_tests {
    use crate::utils::{cross_fold, x_vec};
    use std::collections::HashMap;

    #[test]
    fn x_vec_simple() {
        let v1 = vec![1, 2, 3];
        let v2 = vec![4, 5, 6];

        let result = x_vec(|lhs, rhs| lhs + rhs, v1, v2);

        assert_eq!(result, vec![5, 6, 7, 6, 7, 8, 7, 8, 9]);
    }

    #[test]
    fn cross_fold_single_test() {
        let v = vec![vec![HashMap::from([("k1", "v1")]), HashMap::from([("k2", "v2")])]];

        let result = cross_fold(v);

        assert_eq!(
            result,
            vec![HashMap::from([("k1", "v1")]), HashMap::from([("k2", "v2")])]
        );
    }

    #[test]
    fn cross_fold_multiple_test() {
        let v = vec![
            vec![HashMap::from([("k1", "v1")]), HashMap::from([("k2", "v2")])],
            vec![HashMap::from([("k3", "v3")]), HashMap::from([("k4", "v4")])],
        ];

        let result = cross_fold(v);

        assert_eq!(
            result,
            vec![
                HashMap::from([("k1", "v1"), ("k3", "v3")]),
                HashMap::from([("k1", "v1"), ("k4", "v4")]),
                HashMap::from([("k2", "v2"), ("k3", "v3")]),
                HashMap::from([("k2", "v2"), ("k4", "v4")])
            ]
        );
    }
}
