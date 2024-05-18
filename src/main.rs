use std::cmp::Reverse;
use std::collections::BinaryHeap;

fn main() {
    let mut heap = BinaryHeap::new();
    heap.push(Reverse(5));
    heap.push(Reverse(1));
    heap.push(Reverse(10));
    
    while let Some(Reverse(value)) = heap.pop() {
        println!("{}", value); // 出力: 1, 5, 10
    }
}