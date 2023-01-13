use caesars_palace_nes::*;

fn main() {
    for mask_straight in masks_straight() {
        for index in 0..=0xFF {
            let rng = Rng::with_index(index);
            if can_make_straight(rng, mask_straight) {
                println!("mask: {mask_straight:013b}, index: 0x{index:02X}");
            }
        }
    }
}

fn masks_straight() -> impl Iterator<Item = u32> {
    const MASK_ROYAL: u32 = (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 0);

    (0..=8)
        .map(|shift| 0b11111 << shift)
        .chain(std::iter::once(MASK_ROYAL))
}

fn can_make_straight(mut rng: Rng, mask_straight: u32) -> bool {
    let mut deck = Deck::new();

    let mut mask_cur = 0;
    for _ in 0..5 {
        let card = deck.deal(rng.gen());
        mask_cur |= 1 << card.rank().inner();
    }

    if mask_cur == mask_straight {
        return true;
    }

    // 新しく引くカードたちは全てストレートの構成要素で、かつ distinct でなければならない。
    let mut mask_new = 0;
    for _ in 0..5 {
        let card = deck.deal(rng.gen());
        let bit = 1 << card.rank().inner();
        if (mask_new & bit) != 0 {
            return false;
        }
        if (!mask_straight & bit) != 0 {
            return false;
        }

        mask_new |= bit;
        if ((mask_cur | mask_new) & mask_straight) == mask_straight {
            return true;
        }
    }

    false
}
