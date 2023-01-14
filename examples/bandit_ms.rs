use caesars_palace_nes::*;

fn main() {
    for index in 0..=0xFF {
        let mut rng = Rng::with_index(index);
        let rs: [u8; 3] = std::array::from_fn(|_| rng.gen());

        let prize = bandit_ms_play(rs);

        let factors = prize.factors();
        let factor_sum: u32 = factors.into_iter().sum();

        println!(
            "0x{index:02X}\t{}\t{}\t{}\t{factor_sum}",
            factors[0], factors[1], factors[2]
        );
    }
}
