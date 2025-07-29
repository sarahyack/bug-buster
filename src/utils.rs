use rand::Rng;

// Random Generation Tools

pub fn rand_bool(probability: f32) -> bool {
    let mut rng = rand::rng();
    rng.random::<f32>() < probability
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
