use std::fs::File;
use std::io::{self, Read};
pub fn read_blocks_from_file<T>(path: &str, array: &mut [T], count: usize) -> io::Result<()> {
    let mut file = File::open(path)?;
    let size = size_of::<T>() * count;
    let u8_slice = unsafe { std::slice::from_raw_parts_mut(array.as_mut_ptr() as *mut u8, size) };
    file.read_exact(u8_slice)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_10xi32_from_urandom() {
        let mut array = [0i32; 10];

        let _ = read_blocks_from_file::<i32>("/dev/urandom", &mut array, 10);
        for i in array {
            println!("{}", i);
        }
    }
}
