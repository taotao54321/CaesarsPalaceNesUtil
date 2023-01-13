use caesars_palace_nes::*;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Move {
    index: u8,
    len: usize,
}

fn main() {
    const DEPTH_MAX: u32 = 2;

    for depth in 0..=DEPTH_MAX {
        let mut rng = Rng::new();
        let mut moves = Vec::new();
        solve(&mut rng, depth, &mut moves);
    }
}

fn solve(rng: &mut Rng, depth_remain: u32, moves: &mut Vec<Move>) {
    if depth_remain == 0 {
        solve_leaf(rng, moves);
        return;
    }

    for index in 0..=0xFF {
        for len in 5..=10 {
            moves.push(Move { index, len });

            rng.set_index(index);
            for _ in 0..len {
                rng.gen();
            }
            solve(rng, depth_remain - 1, moves);
            rng.undo(index, len);

            moves.pop().unwrap();
        }
    }
}

fn solve_leaf(rng: &mut Rng, moves: &[Move]) {
    for index in 0..=0xFF {
        rng.set_index(index);
        let (ok, len) = check(rng);
        rng.undo(index, len);

        if ok {
            println!("moves: {moves:?}, index: {index}");
        }
    }
}

/// (ロイヤルフラッシュ可能か, 判定時に消費した乱数の個数) を返す。
fn check(rng: &mut Rng) -> (bool, usize) {
    const MASK_ROYAL: u32 = (1 << 9) | (1 << 10) | (1 << 11) | (1 << 12) | (1 << 0);

    let mut deck = Deck::new();
    let mut len = 0;

    macro_rules! deal {
        () => {{
            let card = deck.deal(rng.gen());
            len += 1;
            card
        }};
    }
    macro_rules! ret {
        ($ok:expr) => {{
            return ($ok, len);
        }};
    }

    // 5 枚引き、各スートの mask を得る。
    let mut masks_cur = [0; 4];
    for _ in 0..5 {
        let card = deal!();
        let i = usize::from(card.suit().inner());
        masks_cur[i] |= 1 << card.rank().inner();
    }

    // いずれかのスートでロイヤルストレートが成立していたらOK。
    if masks_cur.into_iter().any(|mask| mask == MASK_ROYAL) {
        ret!(true);
    }

    // カードを交換する場合、新たに引くカードたちのスートは統一されていなければならない。
    // よって、1 枚目を特別扱いすることで後の処理を簡潔にする。
    let (suit, mask_cur, mut mask_new) = {
        let card = deal!();

        let mask_new = 1 << card.rank().inner();
        if (!MASK_ROYAL & mask_new) != 0 {
            ret!(false);
        }

        let suit = card.suit();
        let i = usize::from(suit.inner());
        let mask_cur = masks_cur[i];

        (suit, mask_cur, mask_new)
    };
    if ((mask_cur | mask_new) & MASK_ROYAL) == MASK_ROYAL {
        ret!(true);
    }

    for _ in 0..4 {
        let card = deal!();

        if card.suit() != suit {
            ret!(false);
        }

        let bit = 1 << card.rank().inner();
        if (mask_new & bit) != 0 {
            ret!(false);
        }
        if (!MASK_ROYAL & bit) != 0 {
            ret!(false);
        }

        mask_new |= bit;
        if ((mask_cur | mask_new) & MASK_ROYAL) == MASK_ROYAL {
            ret!(true);
        }
    }

    ret!(false);
}
