use std::error::Error;
use tokio::sync::mpsc;

pub mod engine;
pub mod parse;

/// Manages the workers and prints out the results. Any errors are passed back to the caller.
pub async fn run() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel(32);
    let file = parse::args()?;

    let parser = parse::input(file, tx);
    let engine = engine::process(rx);

    let (parsed, ledger) = tokio::join!(parser, engine);
    parsed?;

    println!("client, available, held, total, locked");
    for (key, acct) in ledger {
        println!(
            "{}, {}, {}, {}, {}",
            key,
            acct.available,
            acct.held,
            acct.available + acct.held,
            acct.locked
        );
    }

    Ok(())
}
