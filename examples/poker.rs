use clap::Parser;

use caesars_palace_nes::*;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_parser = parse_int::parse::<u8>)]
    index: u8,
}

fn main() {
    let cli = Cli::parse();

    let mut rng = Rng::with_index(cli.index);

    let mut deck = Deck::new();

    for _ in 0..5 {
        print!(" {}", deck.deal(rng.gen()));
    }
    println!();

    for _ in 0..5 {
        print!(" {}", deck.deal(rng.gen()));
    }
    println!();
}
