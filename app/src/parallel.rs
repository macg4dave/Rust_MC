use rayon::prelude::*;

/// Small helper utilities that leverage `rayon` for parallel iteration.
///
/// These helpers are intentionally tiny and well-tested so they are safe
/// to include in the repo as examples of Rayon usage without changing
/// existing behaviour.
pub mod parallel {
    use super::*;

    /// Maps `items` in parallel using the provided closure `f`.
    ///
    /// The closure and element types must be `Send` so they can be executed
    /// across Rayon worker threads.
    pub fn parallel_map<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
    where
        T: Send,
        U: Send,
        F: Fn(T) -> U + Sync + Send,
    {
        items.into_par_iter().map(f).collect()
    }

    /// A small convenience that sums a slice of i64s in parallel.
    pub fn parallel_sum_i64(slice: &[i64]) -> i64 {
        slice.par_iter().cloned().sum()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parallel_map_squares() {
            let input: Vec<i32> = (0..1000).collect();
            let out = parallel_map(input.clone(), |v: i32| v * v);
            let expected: Vec<i32> = input.into_iter().map(|v| v * v).collect();
            assert_eq!(out, expected);
        }

        #[test]
        fn test_parallel_sum_i64() {
            let data: Vec<i64> = (0..10000).map(|v| v as i64).collect();
            let s = parallel_sum_i64(&data);
            let expected: i64 = data.iter().sum();
            assert_eq!(s, expected);
        }
    }
}
