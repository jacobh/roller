use indexmap::IndexMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;

pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

pub fn clamp<T>(x: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if x > min {
        if x > max {
            max
        } else {
            x
        }
    } else {
        min
    }
}
