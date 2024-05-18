use proconio::*;
use rand::prelude::*;
use itertools::Itertools;
use std::{cmp::Reverse, collections::BinaryHeap, process::exit};

pub fn get_time() -> f64 {
    static mut STIME: f64 = -1.0;
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let ms = t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9;
    unsafe {
        if STIME < 0.0 {
            STIME = ms;
        }
        // ローカル環境とジャッジ環境の実行速度差はget_timeで吸収しておくと便利
        #[cfg(feature = "local")]
        {
            (ms - STIME) * 0.8
        }
        #[cfg(not(feature = "local"))]
        {
            ms - STIME
        }
    }
}
/* ⇓ ========== ここから本実装 ========== ⇓ */

struct Input {
    n: usize,
    a: Vec<Vec<i8>>,
}

impl Input {
    fn read_input() -> Self {
        input! {
            n: usize,
            a: [[i8; n]; n],
        }
        Self { n, a }
    }
}

fn write_output(out: &Vec<String>) {
    for s in out {
        println!("{}", s);
    }
}

fn main() {
    let input = Input::read_input();
    let mut ans: Vec<String> = Vec::new();
    for _ in 0..input.n {
        // PRRRRQLLLLPRRRRQLLLLPRRRRQLLLLPRRRRQLLLLPRRRRQ
        ans.push("PRRRRQLLLL".repeat(input.n));
    }
    write_output(&ans);
}