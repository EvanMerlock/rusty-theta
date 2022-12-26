use theta_lang::{repl::{Repl, ReplStatus}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // REPL
    let mut repl = Repl::init();

    // READ IN LINE
    for line in std::io::stdin().lines() {
        let valid_line = line?;
        let resp = repl.line(valid_line)?;

        if let ReplStatus::ReplTerminate = resp {
            break
        }
    }


    Ok(())
}