use std::error::Error;
use tokio::sync::mpsc;

pub mod engine;
pub mod parse;

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
            "   {},\t  {},\t    {},\t  {}\t{}",
            key,
            acct.available,
            acct.held,
            acct.available + acct.held,
            acct.locked
        );
    }

    Ok(())
}
