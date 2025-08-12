#![allow(dead_code)]
// TODO: Create a combat command struct. It's just for the numbers facing side of things, not the
// actual combat. Prob need to create generalized structs and traits for it.
// NOTE: Name should be Joker (it handles all the hit probability and whatnot, so yeah)

use rand::Rng;
use rand::rngs::SmallRng;

#[derive(Copy, Debug, Clone)]
pub enum HitOutcome { Miss, Graze, Hit, Crit }

pub struct HitInputs {
    attacker_acc: f32,
    defender_evasion: f32,
    cover: Option<f32>,
    situational: Option<f32>,
}

impl HitInputs {
    pub fn new(attacker_acc: f32, defender_evasion: f32, cover: Option<f32>, situational: Option<f32>) -> Self {
        HitInputs { attacker_acc, defender_evasion, cover, situational }
    }
}

pub struct DamageInputs {
    pub base_dmg: u32, pub base_hp: u32, pub base_ap: u32,
    pub attacker_dmg_mod: f32,   // trooper/big mod (1.0 = neutral)
    pub outcome_mults: [f32; 4], // per HitOutcome: [miss,graze,hit,crit]
}

impl DamageInputs {
    pub fn new(base_dmg: u32, base_hp: u32, base_ap: u32, attacker_dmg_mod: f32) -> Self {
        let outcome_mults = [0.0, 0.4, 1.0, 1.5];
        DamageInputs { base_dmg, base_hp, base_ap, attacker_dmg_mod, outcome_mults }
    }
}

pub struct AttackContext {
    pub hit: HitInputs,
    pub dmg: DamageInputs,
    pub advantage: i8,          // >0 adv, <0 disadv
    pub clamp_min_max: (f32,f32),
    pub pity_streak: u8,        // consecutive misses
}

impl AttackContext {
    pub fn new(hit: HitInputs, dmg: DamageInputs, advantage: i8, clamp_min_max: (f32, f32), pity_streak: u8) -> Self {
        AttackContext { hit, dmg, advantage, clamp_min_max, pity_streak }
    }
}

pub struct AttackResult {
    pub outcome: HitOutcome,
    pub final_dmg: (u32,u32,u32),
    pub hit_prob_used: f32,
    pub base_p: f32,
    pub pity_lift: f32,
}

pub struct Joker;

impl Joker {
    pub fn new() -> Self {
        Joker
    }

    pub fn hit_probability(h: &HitInputs, scale: f32) -> f32 {
        let atk = h.attacker_acc * h.situational.unwrap_or(1.0);
        let def = h.defender_evasion * h.cover.unwrap_or(1.0);
        let score = atk - def;
        1.0 / (1.0 + (-score / scale).exp())
    }

    pub fn apply_pity(p: f32, streak: u8, step: f32, cap: f32) -> f32 {
        (p + (streak as f32)*step).min(cap)
    }

    pub fn roll_outcome(rng: &mut SmallRng, p: f32) -> HitOutcome {
        let r = (rng.random::<f32>() + rng.random::<f32>()) * 0.5;
        let crit_band = 0.15;
        let graze_band = 0.25;
        let crit_t = p * crit_band;
        let hit_t  = p;
        let graze_t= p + (1.0 - p) * graze_band;

        if r < crit_t      { HitOutcome::Crit  }
        else if r < hit_t  { HitOutcome::Hit   }
        else if r < graze_t{ HitOutcome::Graze }
        else               { HitOutcome::Miss  }
    }

    pub fn resolve( rng: &mut SmallRng, ctx: &AttackContext, scale: f32) -> AttackResult {
        let base_p = Self::hit_probability(&ctx.hit, scale)
            .clamp(ctx.clamp_min_max.0, ctx.clamp_min_max.1);
        let p = Self::apply_pity(base_p, ctx.pity_streak, 0.03, 0.97);

        // advantage/disadvantage by rolling twice and picking best/worst
        let pick = |rng: &mut SmallRng| Self::roll_outcome(rng, p);
        let outcome = if ctx.advantage > 0 {
            let a = pick(rng); let b = pick(rng);
            Self::best(a,b)
        } else if ctx.advantage < 0 {
            let a = pick(rng); let b = pick(rng);
            Self::worst(a,b)
        } else { pick(rng) };

        let mult = match outcome {
            HitOutcome::Miss  => ctx.dmg.outcome_mults[0],
            HitOutcome::Graze => ctx.dmg.outcome_mults[1],
            HitOutcome::Hit   => ctx.dmg.outcome_mults[2],
            HitOutcome::Crit  => ctx.dmg.outcome_mults[3],
        };

        let scale = ctx.dmg.attacker_dmg_mod * mult;
        let to = |x: u32| ((x as f32) * scale) as u32;

        AttackResult {
            outcome,
            final_dmg: (to(ctx.dmg.base_dmg), to(ctx.dmg.base_hp), to(ctx.dmg.base_ap)),
            hit_prob_used: p,
            base_p,
            pity_lift: p - base_p,
        }
    }

    // tiny helpers
    #[inline]
    fn rank(o: HitOutcome) -> u8 { match o { HitOutcome::Miss=>0, HitOutcome::Graze=>1, HitOutcome::Hit=>2, HitOutcome::Crit=>3 } }
    #[inline]
    fn best(a: HitOutcome,b:HitOutcome)->HitOutcome { if Self::rank(a)>=Self::rank(b) {a} else {b} }
    #[inline]
    fn worst(a: HitOutcome,b:HitOutcome)->HitOutcome{ if Self::rank(a)<=Self::rank(b) {a} else {b} }
}
