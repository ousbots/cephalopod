use std::{
    env,
    error::Error,
    fmt,
    fs::File,
    io::{prelude::*, BufReader},
};
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
#[derive(Clone, Debug)]
pub enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl Type {
    /// Parses a string into a transaction type.
    pub fn from(token: &str) -> Result<Self, ParseError> {
        match token {
            "deposit" => Ok(Type::Deposit),
            "withdrawal" => Ok(Type::Withdrawal),
            "dispute" => Ok(Type::Dispute),
            "resolve" => Ok(Type::Resolve),
            "chargeback" => Ok(Type::Chargeback),
            _ => Err(ParseError {
                msg: format!("bad type: {}", token),
            }),
        }
    }
}

/// Transaction data.
#[derive(Clone, Debug)]
pub struct Tx {
    pub typ: Type,
    pub client: u16,
    pub id: u32,
    pub amount: f32,
}

impl Tx {
    /// Parse a record into a transaction struct
    pub fn from(record: &String) -> Result<Self, Box<dyn Error>> {
        let mut tokens = record.split(',');

        Ok(Tx {
            typ: Type::from(tokens.next().ok_or("missing type")?.trim())?,
            client: str::parse::<u16>(tokens.next().ok_or("missing client")?.trim())?,
            id: str::parse::<u32>(tokens.next().ok_or("missing id")?.trim())?,
            amount: str::parse::<f32>(tokens.next().ok_or("missing amount")?.trim())?,
        })
    }
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
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();

    while let Ok(read) = reader.read_line(&mut buffer) {
        if read == 0 {
            break;
        }
        match Tx::from(&buffer) {
            Ok(tx) => {
                if txs.send(tx).await.is_err() {
                    break;
                }
            }
            Err(err) => eprintln!("Couldn't parse record ({}): {}", buffer, err),
        }
        buffer.clear();
    }

    Ok(())
}
