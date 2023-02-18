mod returns;

use std::io::{self, Write};

use clap::Parser;
use returns::{AccumulateArgs, GenReturnsArgs, accumulate, gen_returns};

#[derive(Parser)]
pub struct Args {
    #[command(flatten)]
    gen_returns: GenReturnsArgs,

    #[command(flatten)]
    accumulate: AccumulateArgs,
}

fn main() {
    let args = Args::parse();
    let returns = gen_returns(&args.gen_returns);
    let acc = accumulate(returns, &args.accumulate);
    let stdout = io::stdout();
    let mut handle = io::BufWriter::new(stdout);
    for r in acc.iter() {
        writeln!(handle, "{}", r).unwrap();
    }
    handle.flush().unwrap();
}
