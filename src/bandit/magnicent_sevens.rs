use std::num::NonZeroUsize;

/// スロットマシン "Magnificent Sevens" のプレイ結果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BanditMsPrize {
    factors: [u32; 3],
}

impl BanditMsPrize {
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

/// スロットマシン "Magnificent Sevens" を指定した乱数列でプレイし、結果を返す。
pub fn bandit_ms_play(rs: [u8; 3]) -> BanditMsPrize {
    let reels = randoms_to_reels(rs);

    let factors = std::array::from_fn(|i| {
        const BIASS: [u8; 3] = [2, 0, 4];
        let reels = reels.map(|reel| reel.wrapping_add(BIASS[i]));
        let syms = reels_to_symbols(reels);
        prize_factor(syms)
    });

    BanditMsPrize::new(factors)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Symbol {
    Blank,
    BlackSeven,
    RedSeven,
    SingleBar,
    DoubleBar,
    TripleBar,
}

impl Symbol {
    fn is_seven(self) -> bool {
        use Symbol::*;

        matches!(self, BlackSeven | RedSeven)
    }

    fn is_bar(self) -> bool {
        use Symbol::*;

        matches!(self, SingleBar | DoubleBar | TripleBar)
    }
}

fn randoms_to_reels(rs: [u8; 3]) -> [u8; 3] {
    #[rustfmt::skip]
    const TABLE: [[u8; 66]; 3] = [
        [
            0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
            0x20, 0x22, 0x24, 0x26, 0x28, 0x2A, 0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x1C, 0x1E, 0x18, 0x1A,
            0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A,
            0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x04, 0x06, 0x10, 0x12, 0x14, 0x16, 0x20, 0x22, 0x24, 0x26,
            0x28, 0x2A,
        ],
        [
            0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
            0x20, 0x22, 0x24, 0x26, 0x28, 0x2A, 0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x1C, 0x1E, 0x18, 0x1A,
            0x1C, 0x1E, 0x18, 0x1A, 0x1C, 0x1E, 0x18, 0x0A, 0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A,
            0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x04, 0x06, 0x10, 0x12, 0x24, 0x26, 0x24, 0x26, 0x24, 0x26,
            0x28, 0x2A,
        ],
        [
            0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E,
            0x20, 0x22, 0x24, 0x26, 0x28, 0x2A, 0x00, 0x02, 0x04, 0x06, 0x08, 0x0A, 0x1C, 0x1E, 0x18, 0x1A,
            0x1C, 0x1E, 0x18, 0x1A, 0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A, 0x0C, 0x0E, 0x08, 0x0A,
            0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x04, 0x06, 0x10, 0x12, 0x24, 0x26, 0x14, 0x16, 0x20, 0x22,
            0x28, 0x2A,
        ],
    ];

    const BASE_IDXS: [usize; 3] = [40, 10, 24];

    // 乱数は逆順に適用される。
    std::array::from_fn(|i| {
        let idx = (BASE_IDXS[i] + usize::from(rs[2 - i])) % 66;
        TABLE[i][idx]
    })
}

fn reels_to_symbols(reels: [u8; 3]) -> [Symbol; 3] {
    const TABLE: [Symbol; 22] = {
        use Symbol::*;
        [
            BlackSeven, Blank, DoubleBar, Blank, TripleBar, Blank, RedSeven, Blank, TripleBar,
            Blank, SingleBar, Blank, DoubleBar, Blank, RedSeven, Blank, DoubleBar, Blank,
            SingleBar, Blank, SingleBar, Blank,
        ]
    };

    std::array::from_fn(|i| {
        let idx = usize::from(reels[i] % 44 / 2);
        TABLE[idx]
    })
}

fn prize_factor(syms: [Symbol; 3]) -> u32 {
    use Symbol::*;

    match syms {
        [BlackSeven, BlackSeven, BlackSeven] => 1000,
        _ if syms.into_iter().all(Symbol::is_seven) => 100,
        [TripleBar, TripleBar, TripleBar] => 50,
        [DoubleBar, DoubleBar, DoubleBar] => 20,
        _ if syms.into_iter().all(Symbol::is_bar) => 10,
        _ => 0,
    }
}
