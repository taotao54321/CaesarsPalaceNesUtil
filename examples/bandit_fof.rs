use std::num::NonZeroUsize;

use caesars_palace_nes::*;

fn main() {
    const BET_UNIT: u32 = 1;
    const BET_COUNT: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(3) };

    for index in 0..=0xFF {
        let mut rng = Rng::with_index(index);
        let rs: [u8; 4] = std::array::from_fn(|_| rng.gen());

        let prize = bandit_fof_play(rs);
        let money = prize.calc(BET_UNIT, BET_COUNT);

        println!("0x{index:02X}\t{money}");
    }
}
