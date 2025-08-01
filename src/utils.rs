use rand::{seq::SliceRandom, Rng};

// Macros

#[macro_export]
macro_rules! boost {
    ($stats:expr, $cond:expr, $field:ident += $val:expr) => {
        if $cond {
            $stats.$field += $val;
        }
    };
    ($stats:expr, $cond:expr, $field:ident -= $val:expr) => {
        if $cond {
            $stats.$field = $stats.$field.safe_sub($val);
        }
    };
    ($stats:expr, $cond:expr, $field:ident = $expr:expr) => {
        if $cond {
            $stats.$field = $expr;
        }
    };
}

// Random Generation Tools

pub struct RandBools {}

impl RandBools {
    pub fn rand_bool(probability: f32) -> bool {
        let mut rng = rand::rng();
        rng.random::<f32>() < probability
    }

    pub fn roll_bools<R: Rng>(pool: &mut Vec<&mut bool>, rng: &mut R, max_assign: usize, prob: f32, guaranteed_one: bool) {
        pool.shuffle(rng);
        let mut assigned = 0;
        for (_, item) in pool.into_iter().enumerate() {
            if assigned >= max_assign { break; }
            if Self::rand_bool(prob) || (guaranteed_one && assigned == 0) {
                **item = true;
                assigned += 1;
            }
        }
    }

    pub fn maybe_roll_bools<R: Rng>(
        pool: &mut Vec<&mut bool>,
        rng: &mut R,
        max_assign: usize,
        prob: f32,
        guaranteed_one: bool,
        initial_chance: f32,
    ) {
        if Self::rand_bool(initial_chance) {
            Self::roll_bools(pool, rng, max_assign, prob, guaranteed_one);
        }
    }
}


/// A subtraction that never goes below zero. Created for use subtracting for u32 as well as f32.
pub trait SafeSub {
    fn safe_sub(self, rhs: Self) -> Self;
}

impl SafeSub for u32 {
    fn safe_sub(self, rhs: u32) -> u32 {
        u32::saturating_sub(self, rhs)
    }
}

impl SafeSub for f32 {
    fn safe_sub(self, rhs: f32) -> f32 {
        (self - rhs).max(0.0)
    }
}
