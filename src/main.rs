//--------------------------------------------------------------------
// Description:
// Date: xxxx-xx-xx
//--------------------------------------------------------------------

use anyhow::Result;
use std::process::exit;

mod args;
mod logger;
mod url;

fn real_main() -> Result<()> {
    args::handle()
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("ERROR: {:#}", e);
        exit(1);
    }
}
