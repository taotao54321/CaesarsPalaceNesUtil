use std::fmt::Write as _;
use std::num::NonZeroUsize;

use arrayvec::ArrayVec;

use caesars_palace_nes::*;

const BET_UNIT_BANDIT_FOF: u32 = 1; // $5 を使っても大ジャックポット以外は大差ないので...
const BET_UNIT_BANDIT_MS: u32 = 100;
const BET_UNIT_BANDIT_ROR: u32 = 500;

// $100 を 2 回賭けしたロイヤルフラッシュで 50000 稼げるので一応。
// 多分ロイヤルフラッシュ以外は考慮しなくていいと思う。
const COST_POKER: u32 = 200;

fn main() {
    const MONEY_TARGET: u32 = 0x20000;
    const DEPTH: u32 = 2;

    let mut state = State::new();
    let mut cmoves = ConcreteMoves::new();

    solve(&mut state, &mut cmoves, MONEY_TARGET, DEPTH);
}

fn solve(state: &mut State, cmoves: &mut ConcreteMoves, money_target: u32, depth_remain: u32) {
    if depth_remain == 0 {
        solve_leaf(state, cmoves, money_target);
        return;
    }

    let moves = state.gen_moves();

    for mv in moves {
        for rng_index in 0..=0xFF {
            let (cmv, undo) = state.do_move(mv, rng_index);
            cmoves.push(cmv);
            solve(state, cmoves, money_target, depth_remain - 1);
            cmoves.pop().unwrap();
            state.undo_move(undo);
        }
    }
}

fn solve_leaf(state: &mut State, cmoves: &mut ConcreteMoves, money_target: u32) {
    // 枝刈り。大ジャックポットは時間経過で増えるので、少し余裕を見ておく。
    if state.money() + 100050 < money_target {
        return;
    }

    let moves = state.gen_moves();

    for mv in moves {
        for rng_index in 0..=0xFF {
            let (cmv, undo) = state.do_move(mv, rng_index);
            cmoves.push(cmv);
            if state.money() >= money_target {
                println!("{}", ConcreteMovesPretty::new(cmoves));
            }
            cmoves.pop().unwrap();
            state.undo_move(undo);
        }
    }
}

/// 局面。
#[derive(Debug)]
struct State {
    money: u32,
    rng: Rng,
}

impl Default for State {
    fn default() -> Self {
        Self {
            money: 1000,
            rng: Rng::new(),
        }
    }
}

impl State {
    fn new() -> Self {
        Self::default()
    }

    fn money(&self) -> u32 {
        self.money
    }

    /// 現局面における抽象指し手を列挙する。
    ///
    /// 末端局面に対してもこれを使う。現状特に区別する必要なさそうなので。
    fn gen_moves(&self) -> Moves {
        let mut moves = Moves::new();

        {
            let bet_count_max = 3.min(self.money / BET_UNIT_BANDIT_FOF) as usize;
            if let Some(bet_count_max) = NonZeroUsize::new(bet_count_max) {
                moves.push(Move::BanditFof { bet_count_max });
            }
        }
        {
            let bet_count_max = 3.min(self.money / BET_UNIT_BANDIT_MS) as usize;
            if let Some(bet_count_max) = NonZeroUsize::new(bet_count_max) {
                moves.push(Move::BanditMs { bet_count_max });
            }
        }
        {
            let bet_count_max = 3.min(self.money / BET_UNIT_BANDIT_ROR) as usize;
            if let Some(bet_count_max) = NonZeroUsize::new(bet_count_max) {
                moves.push(Move::BanditRor { bet_count_max });
            }
        }
        {
            if self.money >= COST_POKER {
                for rng_len in 5..=10 {
                    let rng_len = NonZeroUsize::new(rng_len).unwrap();
                    moves.push(Move::Poker { rng_len });
                }
            }
        }

        moves
    }

    /// 乱数インデックスを指定して抽象指し手を実行する。
    fn do_move(&mut self, mv: Move, rng_index: u8) -> (ConcreteMove, UndoInfo) {
        match mv {
            Move::BanditFof { bet_count_max } => self.do_move_bandit_fof(bet_count_max, rng_index),
            Move::BanditMs { bet_count_max } => self.do_move_bandit_ms(bet_count_max, rng_index),
            Move::BanditRor { bet_count_max } => self.do_move_bandit_ror(bet_count_max, rng_index),
            Move::Poker { rng_len } => self.do_move_poker(rng_len, rng_index),
        }
    }

    fn do_move_bandit_fof(
        &mut self,
        bet_count_max: NonZeroUsize,
        rng_index: u8,
    ) -> (ConcreteMove, UndoInfo) {
        const BET_UNIT: u32 = BET_UNIT_BANDIT_FOF;

        debug_assert!(self.money >= BET_UNIT * bet_count_max.get() as u32);

        let rng_index_before = self.rng.index();
        self.rng.set_index(rng_index);

        let rs = rand_4(&mut self.rng);
        let prize = bandit_fof_play(rs);

        // 収入が最大のものを選ぶ。同点なら BET 枚数が最小のものを選ぶ。
        let (bet_count, income) = bet_counts(bet_count_max)
            .map(|bet_count| {
                let gain = prize.calc(BET_UNIT, bet_count) as i32;
                let loss = (BET_UNIT * bet_count.get() as u32) as i32;
                let income = gain - loss;
                (bet_count, income)
            })
            .min_by_key(|&(bet_count, income)| (-income, bet_count))
            .unwrap();

        self.money = self.money.checked_add_signed(income).unwrap();

        let cmv = ConcreteMove::BanditFof {
            rng_index,
            bet_count,
        };
        let undo = UndoInfo {
            rng_index_before,
            rng_index,
            rng_len: 4,
            income,
        };

        (cmv, undo)
    }

    fn do_move_bandit_ms(
        &mut self,
        bet_count_max: NonZeroUsize,
        rng_index: u8,
    ) -> (ConcreteMove, UndoInfo) {
        const BET_UNIT: u32 = BET_UNIT_BANDIT_MS;

        debug_assert!(self.money >= BET_UNIT * bet_count_max.get() as u32);

        let rng_index_before = self.rng.index();
        self.rng.set_index(rng_index);

        let rs = rand_3(&mut self.rng);
        let prize = bandit_ms_play(rs);

        // 収入が最大のものを選ぶ。同点なら BET 枚数が最小のものを選ぶ。
        let (bet_count, income) = bet_counts(bet_count_max)
            .map(|bet_count| {
                let gain = prize.calc(BET_UNIT, bet_count) as i32;
                let loss = (BET_UNIT * bet_count.get() as u32) as i32;
                let income = gain - loss;
                (bet_count, income)
            })
            .min_by_key(|&(bet_count, income)| (-income, bet_count))
            .unwrap();

        self.money = self.money.checked_add_signed(income).unwrap();

        let cmv = ConcreteMove::BanditMs {
            rng_index,
            bet_count,
        };
        let undo = UndoInfo {
            rng_index_before,
            rng_index,
            rng_len: 3,
            income,
        };

        (cmv, undo)
    }

    fn do_move_bandit_ror(
        &mut self,
        bet_count_max: NonZeroUsize,
        rng_index: u8,
    ) -> (ConcreteMove, UndoInfo) {
        const BET_UNIT: u32 = BET_UNIT_BANDIT_ROR;

        debug_assert!(self.money >= BET_UNIT * bet_count_max.get() as u32);

        let rng_index_before = self.rng.index();
        self.rng.set_index(rng_index);

        let rs = rand_3(&mut self.rng);
        let prize = bandit_ror_play(rs);

        // 収入が最大のものを選ぶ。同点なら BET 枚数が最小のものを選ぶ。
        let (bet_count, income) = bet_counts(bet_count_max)
            .map(|bet_count| {
                let gain = prize.calc(BET_UNIT, bet_count) as i32;
                let loss = (BET_UNIT * bet_count.get() as u32) as i32;
                let income = gain - loss;
                (bet_count, income)
            })
            .min_by_key(|&(bet_count, income)| (-income, bet_count))
            .unwrap();

        self.money = self.money.checked_add_signed(income).unwrap();

        let cmv = ConcreteMove::BanditRor {
            rng_index,
            bet_count,
        };
        let undo = UndoInfo {
            rng_index_before,
            rng_index,
            rng_len: 3,
            income,
        };

        (cmv, undo)
    }

    fn do_move_poker(&mut self, rng_len: NonZeroUsize, rng_index: u8) -> (ConcreteMove, UndoInfo) {
        debug_assert!(rng_len.get() >= 5);
        debug_assert!(self.money >= COST_POKER);

        let rng_index_before = self.rng.index();
        self.rng.set_index(rng_index);

        let income = play_poker(&mut self.rng, rng_len);

        self.money = self.money.checked_add_signed(income).unwrap();

        let cmv = ConcreteMove::Poker { rng_index, rng_len };
        let undo = UndoInfo {
            rng_index_before,
            rng_index,
            rng_len: rng_len.get(),
            income,
        };

        (cmv, undo)
    }

    fn undo_move(&mut self, undo: UndoInfo) {
        self.money = self.money.checked_add_signed(-undo.income).unwrap();

        self.rng.undo(undo.rng_index, undo.rng_len);
        self.rng.set_index(undo.rng_index_before);
    }
}

/// 抽象的な指し手。乱数インデックスや実際の BET 枚数の情報を持たない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Move {
    BanditFof { bet_count_max: NonZeroUsize },
    BanditMs { bet_count_max: NonZeroUsize },
    BanditRor { bet_count_max: NonZeroUsize },
    Poker { rng_len: NonZeroUsize },
}

type Moves = ArrayVec<Move, 16>;

/// 具体的な指し手。乱数インデックスや実際の BET 枚数の情報を持つ。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConcreteMove {
    BanditFof {
        rng_index: u8,
        bet_count: NonZeroUsize,
    },
    BanditMs {
        rng_index: u8,
        bet_count: NonZeroUsize,
    },
    BanditRor {
        rng_index: u8,
        bet_count: NonZeroUsize,
    },
    Poker {
        rng_index: u8,
        rng_len: NonZeroUsize,
    },
}

impl ConcreteMove {
    /// `pre`, `self` をホールに戻ることなく連続で実行できるかどうかを返す。
    fn can_fast_forward_from(self, pre: Self) -> bool {
        // 種類が同じで、かつ乱数インデックスの差が適切ならばOK。
        match (self, pre) {
            (
                Self::BanditFof {
                    rng_index: index, ..
                },
                Self::BanditFof {
                    rng_index: index_pre,
                    ..
                },
            ) => rng_index_distance(index_pre, index) == Some(4),
            (
                Self::BanditMs {
                    rng_index: index, ..
                },
                Self::BanditMs {
                    rng_index: index_pre,
                    ..
                },
            ) => rng_index_distance(index_pre, index) == Some(3),
            (
                Self::BanditRor {
                    rng_index: index, ..
                },
                Self::BanditRor {
                    rng_index: index_pre,
                    ..
                },
            ) => rng_index_distance(index_pre, index) == Some(3),
            (
                Self::Poker {
                    rng_index: index, ..
                },
                Self::Poker {
                    rng_index: index_pre,
                    rng_len: len,
                },
            ) => {
                rng_index_distance(index_pre, index).map_or(false, |d| usize::from(d) == len.get())
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for ConcreteMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BanditFof {
                rng_index,
                bet_count,
            } => write!(f, "Fof(0x{rng_index:02X}, {bet_count})"),
            Self::BanditMs {
                rng_index,
                bet_count,
            } => write!(f, "Ms(0x{rng_index:02X}, {bet_count})"),
            Self::BanditRor {
                rng_index,
                bet_count,
            } => write!(f, "Ror(0x{rng_index:02X}, {bet_count})"),
            Self::Poker { rng_index, rng_len } => write!(f, "Poker(0x{rng_index:02X}, {rng_len})"),
        }
    }
}

type ConcreteMoves = ArrayVec<ConcreteMove, 16>;

#[derive(Debug)]
struct ConcreteMovesPretty<'a>(&'a ConcreteMoves);

impl<'a> ConcreteMovesPretty<'a> {
    fn new(cmoves: &'a ConcreteMoves) -> Self {
        Self(cmoves)
    }
}

impl std::fmt::Display for ConcreteMovesPretty<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;

        for i in 0..self.0.len() {
            if i != 0 {
                f.write_str(", ")?;
            }

            // ホールに戻らずに済むケースを検出、表示する。
            if i > 0 && self.0[i].can_fast_forward_from(self.0[i - 1]) {
                f.write_char('*')?;
            }

            self.0[i].fmt(f)?;
        }

        f.write_char(']')?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UndoInfo {
    rng_index_before: u8,
    rng_index: u8,
    rng_len: usize,
    income: i32,
}

/// ポーカーをプレイし、収入を返す。
///
/// ロイヤルフラッシュしか判定していない(手抜き)。
fn play_poker(rng: &mut Rng, rng_len: NonZeroUsize) -> i32 {
    const INCOME_ROYAL: i32 = 50000 - COST_POKER as i32;
    const INCOME_OTHERS: i32 = -(COST_POKER as i32);

    const MASK_ROYAL: u32 = (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 0);

    let mut deck = Deck::new();

    macro_rules! deal {
        () => {{
            deck.deal(rng.gen())
        }};
    }

    // 5 枚引き、各スートの mask を得る。
    let mut masks_cur = [0; 4];
    for _ in 0..5 {
        let card = deal!();
        let i = usize::from(card.suit().inner());
        masks_cur[i] |= 1 << card.rank().inner();
    }

    // 注意: 乱数消費量を一定にするため、ここで全てのカードを引き切らなければならない。
    let exchange_count = rng_len.get() - 5;
    let mut cards_new = ArrayVec::<Card, 5>::new();
    for _ in 0..exchange_count {
        cards_new.push(deal!());
    }

    // いずれかのスートでロイヤルストレートが成立していたらOK。
    if masks_cur.into_iter().any(|mask| mask == MASK_ROYAL) {
        return INCOME_ROYAL;
    }

    // カードを交換しないのなら役なし。
    if exchange_count == 0 {
        return INCOME_OTHERS;
    }

    // カードを交換する場合、新たに引くカードたちのスートは統一されていなければならない。
    // よって、1 枚目を特別扱いすることで後の処理を簡潔にする。
    let (suit, mask_cur, mut mask_new) = {
        let card = cards_new[0];

        let mask_new = 1 << card.rank().inner();
        if (!MASK_ROYAL & mask_new) != 0 {
            return INCOME_OTHERS;
        }

        let suit = card.suit();
        let i = usize::from(suit.inner());
        let mask_cur = masks_cur[i];

        (suit, mask_cur, mask_new)
    };

    for &card in &cards_new[1..] {
        if card.suit() != suit {
            return INCOME_OTHERS;
        }

        let bit = 1 << card.rank().inner();
        if (mask_new & bit) != 0 {
            return INCOME_OTHERS;
        }
        if (!MASK_ROYAL & bit) != 0 {
            return INCOME_OTHERS;
        }

        mask_new |= bit;
        if ((mask_cur | mask_new) & MASK_ROYAL) == MASK_ROYAL {
            return INCOME_ROYAL;
        }
    }

    INCOME_OTHERS
}

fn rand_3(rng: &mut Rng) -> [u8; 3] {
    std::array::from_fn(|_| rng.gen())
}

fn rand_4(rng: &mut Rng) -> [u8; 4] {
    std::array::from_fn(|_| rng.gen())
}

fn bet_counts(bet_count_max: NonZeroUsize) -> impl Iterator<Item = NonZeroUsize> {
    (1..=bet_count_max.get()).map(|x| NonZeroUsize::new(x).unwrap())
}

fn rng_index_distance(src: u8, dst: u8) -> Option<u8> {
    // ホールに戻ることなく乱数インデックスを 250 以上にすることはできない。
    if dst >= 250 {
        return None;
    }

    // src が 250 以上の場合、次は 0 となるので例外処理が必要。
    if src >= 250 {
        return Some(dst + 1);
    }

    Some(if src <= dst {
        dst - src
    } else {
        250 - (src - dst)
    })
}
