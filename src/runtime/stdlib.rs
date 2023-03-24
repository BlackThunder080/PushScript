use std::io::Write;

pub fn putd(n: i64) {
    println!("{n}");
}

pub fn alloc(heap: &mut Vec::<u8>, size: usize) -> u64 {
    let address = heap.len();
    heap.resize(heap.len() + size, 0);
    return address as u64;
}

pub fn write(bytes: &[u8]) {
    std::io::stdout().write_all(bytes).unwrap();
}