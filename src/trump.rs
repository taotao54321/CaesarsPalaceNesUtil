use arrayvec::ArrayVec;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CardSuit {
    Heart = 0,
    Diamond,
    Spade,
    Club,
}

impl CardSuit {
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if inner < 4 {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// `inner` は有効な内部値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        debug_assert!(inner < 4);

        std::mem::transmute(inner)
    }

    pub const fn inner(self) -> u8 {
        self as u8
    }

    pub const fn all() -> [Self; 4] {
        [Self::Heart, Self::Diamond, Self::Spade, Self::Club]
    }
}

impl std::fmt::Display for CardSuit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Heart => "H",
            Self::Diamond => "D",
            Self::Spade => "S",
            Self::Club => "C",
        };
        f.write_str(s)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CardRank(u8);

impl CardRank {
    pub const VALUE_0: Self = unsafe { Self::from_inner_unchecked(0) };
    pub const VALUE_1: Self = unsafe { Self::from_inner_unchecked(1) };
    pub const VALUE_2: Self = unsafe { Self::from_inner_unchecked(2) };
    pub const VALUE_3: Self = unsafe { Self::from_inner_unchecked(3) };
    pub const VALUE_4: Self = unsafe { Self::from_inner_unchecked(4) };
    pub const VALUE_5: Self = unsafe { Self::from_inner_unchecked(5) };
    pub const VALUE_6: Self = unsafe { Self::from_inner_unchecked(6) };
    pub const VALUE_7: Self = unsafe { Self::from_inner_unchecked(7) };
    pub const VALUE_8: Self = unsafe { Self::from_inner_unchecked(8) };
    pub const VALUE_9: Self = unsafe { Self::from_inner_unchecked(9) };
    pub const VALUE_10: Self = unsafe { Self::from_inner_unchecked(10) };
    pub const VALUE_11: Self = unsafe { Self::from_inner_unchecked(11) };
    pub const VALUE_12: Self = unsafe { Self::from_inner_unchecked(12) };

    pub const fn from_inner(inner: u8) -> Option<Self> {
        if inner < 13 {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// `inner` は有効な内部値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        debug_assert!(inner < 13);

        Self(inner)
    }

    pub const fn inner(self) -> u8 {
        self.0
    }

    pub const fn all() -> [Self; 13] {
        [
            Self::VALUE_0,
            Self::VALUE_1,
            Self::VALUE_2,
            Self::VALUE_3,
            Self::VALUE_4,
            Self::VALUE_5,
            Self::VALUE_6,
            Self::VALUE_7,
            Self::VALUE_8,
            Self::VALUE_9,
            Self::VALUE_10,
            Self::VALUE_11,
            Self::VALUE_12,
        ]
    }
}

impl std::fmt::Display for CardRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => f.write_str("A"),
            x @ 1..=9 => write!(f, "{}", x + 1),
            10 => f.write_str("J"),
            11 => f.write_str("Q"),
            12 => f.write_str("K"),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Card(u8);

impl Card {
    pub const fn from_inner(inner: u8) -> Option<Self> {
        if inner < 52 {
            Some(unsafe { Self::from_inner_unchecked(inner) })
        } else {
            None
        }
    }

    /// # Safety
    ///
    /// `inner` は有効な内部値でなければならない。
    pub const unsafe fn from_inner_unchecked(inner: u8) -> Self {
        debug_assert!(inner < 52);

        Self(inner)
    }

    pub const fn from_suit_rank(suit: CardSuit, rank: CardRank) -> Self {
        let inner = 13 * suit.inner() + rank.inner();

        unsafe { Self::from_inner_unchecked(inner) }
    }

    pub const fn suit(self) -> CardSuit {
        unsafe { CardSuit::from_inner_unchecked(self.0 / 13) }
    }

    pub const fn rank(self) -> CardRank {
        unsafe { CardRank::from_inner_unchecked(self.0 % 13) }
    }

    pub const fn all() -> [Self; 52] {
        macro_rules! card {
            ($suit:ident, $rank_inner:expr) => {{
                let rank = unsafe { CardRank::from_inner_unchecked($rank_inner) };
                Card::from_suit_rank(CardSuit::$suit, rank)
            }};
        }

        [
            card!(Heart, 0),
            card!(Heart, 1),
            card!(Heart, 2),
            card!(Heart, 3),
            card!(Heart, 4),
            card!(Heart, 5),
            card!(Heart, 6),
            card!(Heart, 7),
            card!(Heart, 8),
            card!(Heart, 9),
            card!(Heart, 10),
            card!(Heart, 11),
            card!(Heart, 12),
            card!(Diamond, 0),
            card!(Diamond, 1),
            card!(Diamond, 2),
            card!(Diamond, 3),
            card!(Diamond, 4),
            card!(Diamond, 5),
            card!(Diamond, 6),
            card!(Diamond, 7),
            card!(Diamond, 8),
            card!(Diamond, 9),
            card!(Diamond, 10),
            card!(Diamond, 11),
            card!(Diamond, 12),
            card!(Spade, 0),
            card!(Spade, 1),
            card!(Spade, 2),
            card!(Spade, 3),
            card!(Spade, 4),
            card!(Spade, 5),
            card!(Spade, 6),
            card!(Spade, 7),
            card!(Spade, 8),
            card!(Spade, 9),
            card!(Spade, 10),
            card!(Spade, 11),
            card!(Spade, 12),
            card!(Club, 0),
            card!(Club, 1),
            card!(Club, 2),
            card!(Club, 3),
            card!(Club, 4),
            card!(Club, 5),
            card!(Club, 6),
            card!(Club, 7),
            card!(Club, 8),
            card!(Club, 9),
            card!(Club, 10),
            card!(Club, 11),
            card!(Club, 12),
        ]
    }
}

impl std::fmt::Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.suit().fmt(f)?;
        self.rank().fmt(f)?;

        Ok(())
    }
}

type DeckCards = ArrayVec<Card, 52>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deck {
    cards: DeckCards,
}

impl Default for Deck {
    fn default() -> Self {
        Self {
            cards: Self::cards_default(),
        }
    }
}

impl Deck {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn deal(&mut self, r: u8) -> Card {
        if self.cards.is_empty() {
            self.cards = Self::cards_default();
        }

        let idx = usize::from(r) % self.cards.len();
        self.cards.remove(idx)
    }

    fn cards_default() -> DeckCards {
        DeckCards::from(Card::all())
    }
}
