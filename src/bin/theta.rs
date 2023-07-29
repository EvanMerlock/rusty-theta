use log::error;
use theta_lang::repl::{Repl, ReplStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // REPL
    let mut repl = Repl::init();

    // READ IN LINE
    for line in std::io::stdin().lines() {
        let valid_line = line?;
        let resp = match repl.line(valid_line) {
            Ok(repl_status) => repl_status,
            Err(e) => {
                error!("REPL evaluation failed: {}, resetting VM", e);
                repl = Repl::init();
                continue;
            },
        };

        if let ReplStatus::ReplTerminate = resp {
            break
        }
    }


    Ok(())
}