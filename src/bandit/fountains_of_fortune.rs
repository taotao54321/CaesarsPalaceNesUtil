use std::num::NonZeroUsize;

/// スロットマシン "Fountains of Fortune" のプレイ結果。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BanditFofPrize {
    Null,
    Normal { rank: usize },
    AllGreenSeven,
    AllBlackSeven,
}

impl BanditFofPrize {
    pub fn calc(self, bet_unit: u32, bet_count: NonZeroUsize) -> u32 {
        const FACTOR_TABLE: [[u32; 3]; 9] = [
            [2, 4, 6],
            [5, 10, 15],
            [10, 20, 30],
            [10, 20, 30],
            [20, 40, 60],
            [100, 200, 300],
            [200, 400, 0],
            [2000, 5000, 0],
            [100, 200, 300],
        ];

        match self {
            Self::Null => 0,
            Self::Normal { rank } => {
                let factor = FACTOR_TABLE[rank][bet_count.get() - 1];
                bet_unit * factor
            }
            Self::AllGreenSeven => {
                const FACTORS: [u32; 2] = [200, 400];
                if bet_count.get() == 3 {
                    // NOTE: 小ジャックポットの最小金額。実際には時間とともに少しずつ増える。
                    750
                } else {
                    bet_unit * FACTORS[bet_count.get() - 1]
                }
            }
            Self::AllBlackSeven => {
                const FACTORS: [u32; 2] = [2000, 5000];
                if bet_count.get() == 3 {
                    // NOTE: 大ジャックポットの最小金額。実際には時間とともに少しずつ増える。
                    100000
                } else {
                    bet_unit * FACTORS[bet_count.get() - 1]
                }
            }
        }
    }
}

/// スロットマシン "Fountains of Fortune" を指定した乱数列でプレイし、結果を返す。
pub fn bandit_fof_play(rs: [u8; 4]) -> BanditFofPrize {
    let reels = randoms_to_reels(rs);
    let reels = reels.map(|reel| reel + 2);
    let syms = reels_to_symbols(reels);
    calc_prize(syms)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Symbol {
    Null,
    Cherry,
    RedSeven,
    GreenSeven,
    BlackSeven,
}

impl Symbol {
    pub fn is_seven(self) -> bool {
        use Symbol::*;

        matches!(self, RedSeven | GreenSeven | BlackSeven)
    }
}

fn randoms_to_reels(rs: [u8; 4]) -> [u8; 4] {
    #[rustfmt::skip]
    const TABLES: [[u8; 64]; 2] = [
        [
            0x00, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x03, 0x03, 0x03, 0x03, 0x04, 0x04, 0x05, 0x05, 0x05,
            0x05, 0x06, 0x06, 0x07, 0x07, 0x07, 0x07, 0x08, 0x08, 0x09, 0x09, 0x09, 0x09, 0x0A, 0x0A, 0x0B,
            0x0B, 0x0B, 0x0B, 0x0C, 0x0C, 0x0D, 0x0D, 0x0D, 0x0D, 0x0E, 0x0E, 0x0F, 0x0F, 0x0F, 0x0F, 0x10,
            0x10, 0x11, 0x11, 0x11, 0x11, 0x12, 0x12, 0x13, 0x13, 0x13, 0x13, 0x14, 0x14, 0x15, 0x15, 0x15,
        ],
        [
            0x00, 0x01, 0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x03, 0x03, 0x03, 0x04, 0x04, 0x05, 0x05, 0x05,
            0x05, 0x06, 0x06, 0x07, 0x07, 0x07, 0x07, 0x08, 0x08, 0x09, 0x09, 0x09, 0x09, 0x0A, 0x0A, 0x0B,
            0x0B, 0x0B, 0x0B, 0x0C, 0x0C, 0x0D, 0x0D, 0x0D, 0x0D, 0x0E, 0x0E, 0x0F, 0x0F, 0x0F, 0x0F, 0x10,
            0x10, 0x11, 0x11, 0x11, 0x11, 0x12, 0x12, 0x13, 0x13, 0x13, 0x13, 0x14, 0x14, 0x15, 0x15, 0x15,
        ],
    ];

    const BASE_IDXS: [usize; 4] = [18, 10, 2, 16];

    // 乱数は逆順に適用される。
    std::array::from_fn(|i| {
        let j = [0, 1, 1, 0][i];
        let idx = (BASE_IDXS[i] + usize::from(rs[3 - i])) % 64;
        TABLES[j][idx]
    })
}

fn reels_to_symbols(reels: [u8; 4]) -> [Symbol; 4] {
    const TABLES: [[Symbol; 11]; 2] = {
        use Symbol::*;
        [
            [
                BlackSeven, GreenSeven, GreenSeven, GreenSeven, RedSeven, GreenSeven, GreenSeven,
                GreenSeven, Cherry, GreenSeven, GreenSeven,
            ],
            [
                BlackSeven, RedSeven, RedSeven, RedSeven, GreenSeven, RedSeven, RedSeven, RedSeven,
                Cherry, RedSeven, RedSeven,
            ],
        ]
    };

    std::array::from_fn(|i| {
        let reel = reels[i];
        if reel % 2 == 0 {
            let j = [0, 1, 1, 0][i];
            let idx = usize::from(reel % 22 / 2);
            TABLES[j][idx]
        } else {
            Symbol::Null
        }
    })
}

fn calc_prize(syms: [Symbol; 4]) -> BanditFofPrize {
    use Symbol::*;

    // 黒 7 による役。
    {
        let count = syms.into_iter().filter(|&sym| sym == BlackSeven).count();
        if count == 4 {
            return BanditFofPrize::AllBlackSeven;
        } else if count == 3 {
            return BanditFofPrize::Normal { rank: 5 };
        }
    }

    // 緑 7 による役。
    {
        let count = syms.into_iter().filter(|&sym| sym == GreenSeven).count();
        if count == 4 {
            return BanditFofPrize::AllGreenSeven;
        } else if count == 3 {
            return BanditFofPrize::Normal { rank: 4 };
        }
    }

    // 赤 7 による役。
    {
        let count = syms.into_iter().filter(|&sym| sym == RedSeven).count();
        if count == 4 {
            return BanditFofPrize::Normal { rank: 5 };
        } else if count == 3 {
            return BanditFofPrize::Normal { rank: 3 };
        }
    }

    // 任意の 7 による役。
    {
        let count = syms.into_iter().filter(|&sym| sym.is_seven()).count();
        if count == 4 {
            return BanditFofPrize::Normal { rank: 3 };
        } else if count == 3 {
            return BanditFofPrize::Normal { rank: 1 };
        }
    }

    // チェリーによる役。
    {
        let count = syms.into_iter().filter(|&sym| sym == Cherry).count();
        match count {
            4 => return BanditFofPrize::Normal { rank: 4 },
            3 => return BanditFofPrize::Normal { rank: 2 },
            2 => return BanditFofPrize::Normal { rank: 1 },
            1 => return BanditFofPrize::Normal { rank: 0 },
            _ => {}
        }
    }

    BanditFofPrize::Null
}
