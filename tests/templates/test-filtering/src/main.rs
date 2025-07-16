// Source code file - should be included in filtering
fn main() {
    println!("Hello from test-filtering template!");

    // This file tests that source code is properly included
    let message = "This is a Rust source file";
    println!("{}", message);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }
}
