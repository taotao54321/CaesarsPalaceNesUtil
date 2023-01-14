use std::num::NonZeroUsize;

/// スロットマシン "Riches of Rome" のプレイ結果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BanditRorPrize {
    factors: [u32; 3],
}

impl BanditRorPrize {
    fn new(factors: [u32; 3]) -> Self {
        Self { factors }
    }

    /// 各行の賞金倍率を返す。(中央行, 上行, 下行) の順 (BET 順と同じ)。
    pub fn factors(self) -> [u32; 3] {
        self.factors
    }

    pub fn calc(self, bet_unit: u32, bet_count: NonZeroUsize) -> u32 {
        self.factors
            .into_iter()
            .take(bet_count.get())
            .map(|factor| bet_unit * factor)
            .sum()
    }
}

/// スロットマシン "Riches of Rome" を指定した乱数列でプレイし、結果を返す。
pub fn bandit_ror_play(rs: [u8; 3]) -> BanditRorPrize {
    let reels = randoms_to_reels(rs);

    let factors = std::array::from_fn(|i| {
        const BIASS: [u8; 3] = [2, 0, 4];
        let reels = reels.map(|reel| reel.wrapping_add(BIASS[i]));
        let syms = reels_to_symbols(reels);
        prize_factor(syms)
    });

    BanditRorPrize::new(factors)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Symbol {
    Watermelon,
    Lemon,
    Bar,
    Seven,
    Cherry,
    Orange,
    Plum,
    Bell,
}

fn randoms_to_reels(rs: [u8; 3]) -> [u8; 3] {
    const BASES: [u8; 3] = [40, 10, 24];

    // 乱数は逆順に適用される。
    std::array::from_fn(|i| 2 * (BASES[i].wrapping_add(rs[2 - i]) % 20))
}

fn reels_to_symbols(reels: [u8; 3]) -> [Symbol; 3] {
    #[rustfmt::skip]
    const TABLE: [[Symbol; 20]; 3] = {
        use Symbol::*;
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

fn prize_factor(syms: [Symbol; 3]) -> u32 {
    use Symbol::*;

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
