/// スロットマシン "Riches of Rome" を指定した乱数列でプレイし、各行の賞金倍率を返す。
///
/// 戻り値は (中央行, 上行, 下行) の順 (BET 順と同じ)。
pub fn bandit_ror_play(rs: [u8; 3]) -> [u32; 3] {
    let reels = bandit_ror_reels(rs);

    std::array::from_fn(|i| {
        const BIASS: [u8; 3] = [2, 0, 4];
        let reels = reels.map(|reel| reel.wrapping_add(BIASS[i]));
        let syms = bandit_ror_reels_to_symbols(reels);
        bandit_ror_prize_factor(syms)
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BanditRorSymbol {
    Watermelon,
    Lemon,
    Bar,
    Seven,
    Cherry,
    Orange,
    Plum,
    Bell,
}

fn bandit_ror_reels(rs: [u8; 3]) -> [u8; 3] {
    const BASES: [u8; 3] = [40, 10, 24];

    // 乱数は逆順に適用される。
    std::array::from_fn(|i| 2 * (BASES[i].wrapping_add(rs[2 - i]) % 20))
}

fn bandit_ror_reels_to_symbols(reels: [u8; 3]) -> [BanditRorSymbol; 3] {
    #[rustfmt::skip]
    const TABLE: [[BanditRorSymbol; 20]; 3] = {
        use BanditRorSymbol::*;
        [
            [
                Orange, Watermelon, Plum,   Cherry, Plum,       Orange, Seven,  Bell, Orange, Cherry,
                Bar,    Plum,       Orange, Plum,   Watermelon, Plum,   Orange, Plum, Bar,    Plum,
            ],
            [
                Cherry, Plum,   Cherry, Seven, Cherry, Bell, Bar,    Bell,   Cherry, Orange,
                Bell,   Orange, Plum,   Bell,  Cherry, Bar,  Orange, Cherry, Bell,   Watermelon,
            ],
            [
                Lemon, Orange, Plum, Bell, Orange, Lemon, Bar,  Watermelon, Bell,  Plum,
                Lemon, Bell,   Plum, Bell, Seven,  Lemon, Bell, Orange,     Lemon, Bar,
            ],
        ]
    };

    std::array::from_fn(|i| {
        let idx = usize::from(reels[i] % 40 / 2);
        TABLE[i][idx]
    })
}

fn bandit_ror_prize_factor(syms: [BanditRorSymbol; 3]) -> u32 {
    use BanditRorSymbol::*;

    match syms {
        [Seven, Seven, Seven] => 200,
        [Bar, Bar, Bar] => 100,
        [Watermelon, Watermelon, Bar | Watermelon] => 100,
        [Bell, Bell, Bar | Bell] => 18,
        [Plum, Plum, Bar | Plum] => 14,
        [Orange, Orange, Bar | Orange] => 10,
        [Cherry, Cherry, _] => 5,
        [Cherry, _, _] => 2,
        _ => 0,
    }
}
