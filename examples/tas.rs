use std::fmt::Write as _;
use std::num::NonZeroUsize;

use arrayvec::ArrayVec;

use caesars_palace_nes::*;

const BET_UNIT_BANDIT_FOF: u32 = 1; // $5 を使っても大ジャックポット以外は大差ないので...
const BET_UNIT_BANDIT_MS: u32 = 100;
const BET_UNIT_BANDIT_ROR: u32 = 500;

fn main() {
    let mut state = State::new();
    let mut cmoves = ConcreteMoves::new();

    solve(&mut state, &mut cmoves, 0x20000, 3);
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

    let moves = state.gen_leaf_moves();

    for mv in moves {
        for rng_index in 0..=0xFF {
            let (cmv, undo) = state.do_move(mv, rng_index);
            cmoves.push(cmv);
            if state.money() >= money_target {
                println!("{}", ConcreteMovesPretty::new(&cmoves));
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

        moves
    }

    /// 現局面を末端局面とみなして抽象指し手を列挙する。
    ///
    /// NOTE: 今のところ `gen_moves()` と同じだが、ポーカーを利用した乱数調整なども考えられるので...。
    fn gen_leaf_moves(&self) -> Moves {
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

        moves
    }

    /// 乱数インデックスを指定して抽象指し手を実行する。
    fn do_move(&mut self, mv: Move, rng_index: u8) -> (ConcreteMove, UndoInfo) {
        match mv {
            Move::BanditFof { bet_count_max } => self.do_move_bandit_fof(bet_count_max, rng_index),
            Move::BanditMs { bet_count_max } => self.do_move_bandit_ms(bet_count_max, rng_index),
            Move::BanditRor { bet_count_max } => self.do_move_bandit_ror(bet_count_max, rng_index),
        }
    }

    fn do_move_bandit_fof(
        &mut self,
        bet_count_max: NonZeroUsize,
        rng_index: u8,
    ) -> (ConcreteMove, UndoInfo) {
        const BET_UNIT: u32 = BET_UNIT_BANDIT_FOF;

        debug_assert!(self.money >= BET_UNIT * bet_count_max.get() as u32);

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
            rng_index,
            rng_len: 3,
            income,
        };

        (cmv, undo)
    }

    fn undo_move(&mut self, undo: UndoInfo) {
        self.money = self.money.checked_add_signed(-undo.income).unwrap();

        self.rng.undo(undo.rng_index, undo.rng_len);
    }
}

/// 抽象的な指し手。乱数インデックスや実際の BET 枚数の情報を持たない。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Move {
    BanditFof { bet_count_max: NonZeroUsize },
    BanditMs { bet_count_max: NonZeroUsize },
    BanditRor { bet_count_max: NonZeroUsize },
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
}

impl std::fmt::Display for ConcreteMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BanditFof {
                rng_index,
                bet_count,
            } => write!(f, "Fof(0x{:02X}, {})", rng_index, bet_count),
            Self::BanditMs {
                rng_index,
                bet_count,
            } => write!(f, "Ms(0x{:02X}, {})", rng_index, bet_count),
            Self::BanditRor {
                rng_index,
                bet_count,
            } => write!(f, "Ror(0x{:02X}, {})", rng_index, bet_count),
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

        let mut first = true;
        for cmv in self.0 {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            cmv.fmt(f)?;
        }

        f.write_char(']')?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UndoInfo {
    rng_index: u8,
    rng_len: usize,
    income: i32,
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
