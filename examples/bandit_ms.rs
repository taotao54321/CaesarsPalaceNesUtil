use caesars_palace_nes::*;

fn main() {
    for index in 0..=0xFF {
        let mut rng = Rng::with_index(index);
        let rs: [u8; 3] = std::array::from_fn(|_| rng.gen());

        let prizes = bandit_ms_play(rs);
        let prize_sum: u32 = prizes.into_iter().sum();

        println!(
            "0x{index:02X}\t{}\t{}\t{}\t{prize_sum}",
            prizes[0], prizes[1], prizes[2]
        );
    }
}
