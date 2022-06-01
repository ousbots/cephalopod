use std::process;

#[tokio::main]
async fn main() {
    match cephalopod::run().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Fatal Error: {}", err);
            process::exit(1);
        }
    }
}
