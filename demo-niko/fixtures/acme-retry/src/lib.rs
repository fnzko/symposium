pub fn with_retries<T, E>(attempts: u32, mut op: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    let mut last = op();
    for _ in 1..attempts {
        if last.is_ok() {
            break;
        }
        last = op();
    }
    last
}
