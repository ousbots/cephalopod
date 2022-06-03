use csv::ReaderBuilder;
use serde::Deserialize;
use std::{env, error::Error, fmt, fs::File, io::BufReader};
use tokio::sync::mpsc;

/// Custom error type to pass error messages.
#[derive(Debug)]
pub struct ParseError {
    msg: String,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// Transaction type.
#[derive(Clone, Debug, Deserialize)]
pub enum Type {
    #[serde(rename(deserialize = "deposit"))]
    Deposit,
    #[serde(rename(deserialize = "withdrawal"))]
    Withdrawal,
    #[serde(rename(deserialize = "dispute"))]
    Dispute,
    #[serde(rename(deserialize = "resolve"))]
    Resolve,
    #[serde(rename(deserialize = "chargeback"))]
    Chargeback,
}

/// Transaction data.
#[derive(Clone, Debug, Deserialize)]
pub struct Tx {
    #[serde(rename(deserialize = "type"))]
    pub typ: Type,
    pub client: u16,
    #[serde(rename(deserialize = "tx"))]
    pub id: u32,
    pub amount: f32,
}

/// Parses the command line arguments and returns the path passed in.
pub fn args() -> Result<String, ParseError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        if args.len() > 2 {
            eprintln!("Too many arguments!");
        } else {
            eprintln!("Missing path!");
        }
        eprint!("USAGE:\n\t{} [path]\n", args[0]);

        return Err(ParseError {
            msg: String::from("incorrect number of arguments"),
        });
    }

    Ok(args[1].clone())
}

/// Parses the input file into transactions, returning a vector of transactions.
pub async fn input(path: String, txs: mpsc::Sender<Tx>) -> Result<(), Box<dyn Error>> {
    let file = File::open(path).unwrap();
    let mut reader = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(BufReader::new(file));

    for record in reader.deserialize() {
        let tx: Tx = record?;
        if txs.send(tx).await.is_err() {
            break;
        }
    }

    Ok(())
}
