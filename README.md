# under_dev
Macros to indicate under development functions

# Usage
## unimplemented_functions!
```rust
unimplemented_functions! {
    pub fn function1(a: usize) {}

    pub fn function2(b: i32) -> std::io::Result<usize> {
        return Ok(1usize)
    }

    fn private_function() {}
}
```

## wip
```rust
#[wip("Comment about the implementation status of this function")]
pub fn function(c: &str) -> i32 { 0i32 }
```
