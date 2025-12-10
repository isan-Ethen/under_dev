use under_dev::{unimplemented_functions, wip};

#[wip]
fn my_wip_function() {
    // unimplemented!()
}

#[wip("Need to check EINVAL")]
pub fn public_wip(a: i32) -> i32 {
    // ...
    a
}

#[wip(ffi = true)]
unsafe extern "C" fn ffi_wip() {
    // ...
}

unimplemented_functions! {
    ffi = true,
    pub fn sys_open(path: *const u8, flags: i32) -> i32 {}
    fn sys_read(fd: i32, buf: *mut u8, count: usize) -> isize {}
}

fn main() {
    my_wip_function();
    println!("Macro compilation test not successful.");
}
