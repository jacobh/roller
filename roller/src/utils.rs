use async_std::prelude::*;
use futures::stream::{self, StreamExt};
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use std::hash::BuildHasherDefault;
use std::time::{Duration, Instant};

pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

pub fn tick_stream(interval: Duration) -> impl Stream<Item = ()> {
    let mut next_tick_at = Instant::now();

    stream::repeat(()).then(move |()| {
        let until = next_tick_at;
        next_tick_at += interval;
        let now = Instant::now();
        async_std::task::sleep(if now < until {
            until - now
        } else {
            Duration::from_secs(0)
        })
    })
}

pub fn shift_remove_vec<T>(vec: &mut Vec<T>, item: &T) -> Option<T>
where
    T: PartialEq,
{
    let item_idx = vec.iter().position(|x| item == x);

    if let Some(item_idx) = item_idx {
        Some(vec.remove(item_idx))
    } else {
        None
    }
}
