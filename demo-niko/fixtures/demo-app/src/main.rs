use anyhow::Result;

fn main() -> Result<()> {
    let attempts = acme_retry::with_retries(3, || Ok::<_, anyhow::Error>(42))?;
    println!("answer: {attempts}");
    Ok(())
}
