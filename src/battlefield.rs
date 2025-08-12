#![allow(dead_code, unused_variables)]
// TODO: Implement the turn_handler
// TODO: Implement effect application

use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::log;
use crate::debug::LOG;
use crate::utils::{RngHub, SafeSub};
use crate::hive::Cartographer;
use crate::bugs::{Broodmother, Bug, };
use crate::troopers::{Commander, Trooper};
use crate::combat::{Joker, HitOutcome, HitInputs, DamageInputs, AttackContext};

enum Combatant<'a> {
    Trooper(&'a Trooper),
    Bug(&'a Bug),
}

impl<'a> Combatant<'a> {
    fn accuracy(&self) -> f32 {
        match self {
            Combatant::Trooper(t) => t.accuracy(),
            Combatant::Bug(b)     => b.accuracy(),
        }
    }
    fn agility(&self) -> f32 {
        match self {
            Combatant::Trooper(t) => t.agility(),
            Combatant::Bug(b)     => b.agility(),
        }
    }
    /// Returns (base_dmg, base_hp_dmg, base_ap_dmg)
    fn damage_tuple(&self) -> (u32, u32, u32) {
        match self {
            Combatant::Trooper(t) => t.damage(),
            Combatant::Bug(b)     => b.damage(),
        }
    }
    /// Optional: attacker-side dmg modifier (Troopers have class dmg_mod, bugs maybe 1.0)
    fn dmg_mod(&self) -> f32 {
        match self {
            Combatant::Trooper(t) => 1.0,
            Combatant::Bug(_b)    => 1.0,
        }
    }
}

#[derive(Default)]
pub struct RollStats {
    attempts: u32,
    miss: u32,
    graze: u32,
    hit: u32,
    crit: u32,
    p_sum: f32,
}

impl RollStats {
    fn record(&mut self, o: HitOutcome, p: f32) {
        self.attempts += 1;
        self.p_sum += p;
        match o {
            HitOutcome::Miss => self.miss += 1,
            HitOutcome::Graze => self.graze += 1,
            HitOutcome::Hit => self.hit += 1,
            HitOutcome::Crit => self.crit += 1,
        }
    }
    fn summary(&self, label: &str) -> String {
        let n = self.attempts.max(1) as f32;
        format!(
            "{label}: n={} | p̄={:.3} | Miss {:.1}%  Graze {:.1}%  Hit {:.1}%  Crit {:.1}%",
            self.attempts,
            self.p_sum / n,
            100.0 * self.miss as f32 / n,
            100.0 * self.graze as f32 / n,
            100.0 * self.hit as f32 / n,
            100.0 * self.crit as f32 / n,
        )
    }
}

#[derive(Default)]
pub struct PityStats {
    total: u32,
    used: u32,            // streak > 0
    sum_lift: f32,        // Σ (p - base_p)
    sum_streak: u32,      // Σ streak (when used)
    max_streak: u8,
    broke_streak: u32,    // pity active & outcome was Hit/Crit
}

impl PityStats {
    fn record(&mut self, base_p: f32, p: f32, streak: u8, outcome: HitOutcome) {
        self.total += 1;
        if streak > 0 {
            self.used += 1;
            self.sum_lift += p - base_p;
            self.sum_streak += streak as u32;
            self.max_streak = self.max_streak.max(streak);
            if matches!(outcome, HitOutcome::Hit | HitOutcome::Crit) {
                self.broke_streak += 1;
            }
        }
    }

    fn summary(&self, label: &str) -> String {
        let usedf = self.used as f32;
        let use_rate = if self.total == 0 { 0.0 } else { 100.0 * (self.used as f32) / (self.total as f32) };
        let avg_lift = if self.used == 0 { 0.0 } else { self.sum_lift / usedf };
        let avg_streak = if self.used == 0 { 0.0 } else { self.sum_streak as f32 / usedf };
        let broke_pct = if self.used == 0 { 0.0 } else { 100.0 * (self.broke_streak as f32) / usedf };
        format!(
            "{label}: used_on={} ({use_rate:.1}%) | avg_lift=+{avg_lift:.3} | avg_streak={avg_streak:.2} | max_streak={} | broke_streak={} ({broke_pct:.1}%)",
            self.used, self.max_streak, self.broke_streak
        )
    }
}

#[derive(Clone, Copy)]
pub struct SimOpts {
    pub clamp: (f32, f32),
    pub scale: f32,
    pub round_cap: usize,
    pub adv_trooper: i8,
    pub adv_bug: i8,
    pub rebalance_hp: f32,
    pub rebalance_ap: f32,
    pub rebalance_dmg: f32,
}

impl Default for SimOpts {
    fn default() -> Self {
        Self {
            clamp: (0.05, 0.95),
            scale: 0.60,
            round_cap: 50,
            adv_trooper: 0,
            adv_bug: 0,
            rebalance_hp: 1.55,
            rebalance_ap: 1.55,
            rebalance_dmg: 0.85,
        }
    }
}

pub struct WaveSummary {
    pub rounds: usize,
    pub trooper_alive: usize,
    pub bug_alive: usize,
    pub trooper_rolls: RollStats,
    pub bug_rolls: RollStats,
    pub trooper_pity: PityStats,
    pub bug_pity: PityStats,
}

pub struct CampaignSummary {
    pub waves_cleared: usize,
    pub last_wave: WaveSummary,
}

impl CampaignSummary {
    pub fn summary(&self) {
        log!(info, format!("⚔️ Waves Cleared: {:?}", self.waves_cleared), true);
    }
}

pub struct Overwatch {
    turn: usize,
    master_rng: SmallRng,
    cartographer: Cartographer,
    commander: Commander,
    broodmother: Broodmother,
    joker: Joker,
}

impl Overwatch {
    pub fn new() -> Self {
        let turn = 0;
        let hub = RngHub::new(None);
        hub.log_master_seed();
        let master_seed = hub.master_seed;
        let master_rng = SmallRng::seed_from_u64(master_seed);
        let cartographer = Cartographer::new();
        let commander = Commander::new(3);
        let broodmother = Broodmother::new();
        let joker = Joker::new();

        Overwatch {
            turn,
            master_rng,
            cartographer,
            commander,
            broodmother,
            joker
        }
    }

    fn build_hit_inputs(attacker: &Combatant, defender: &Combatant) -> HitInputs {
        HitInputs::new(attacker.accuracy(), defender.agility(), None, None)
    }

    fn build_dmg_inputs(attacker: &Combatant) -> DamageInputs {
        let (dmg, hp_dmg, ap_dmg) = attacker.damage_tuple();
        DamageInputs::new(dmg, hp_dmg, ap_dmg, attacker.dmg_mod())
    }

    fn build_context(attacker: Combatant, defender: Combatant, advantage: i8, clamp_min_max: (f32, f32), pity_streak: u8) -> AttackContext {
        AttackContext::new(Self::build_hit_inputs(&attacker, &defender), Self::build_dmg_inputs(&attacker), advantage, clamp_min_max, pity_streak)
    }

    fn any_trooper_alive(&self) -> bool {
        self.commander.team.iter().any(|t| t.is_alive())
    }

    fn any_bug_alive(wave: &[Bug]) -> bool {
        wave.iter().any(|b| b.is_alive())
    }

    fn first_alive_trooper_idx(&self) -> Option<usize> {
        self.commander.team.iter().position(|t| t.is_alive())
    }

    fn first_alive_bug_idx(wave: &[Bug]) -> Option<usize> {
        wave.iter().position(|b| b.is_alive())
    }

    fn rebalance(&mut self, wave: &mut [Bug], hp_factor: f32, ap_factor: f32, dmg_factor: f32) {
        self.commander.rebalance_team(hp_factor, ap_factor);
        self.broodmother.rebalance_wave(wave, dmg_factor);
    }

    pub fn start_game(&mut self) {
        self.commander.spawn_troopers(&self.commander.team);
        self.commander.log_team_gear(&self.commander.team);

        self.cartographer.spawn_chambers(5);

        // let wave = self.broodmother.spawn_test_wave(5);
        let count: usize = 5;
        let waves: Vec<Vec<Bug>> = (0..count)
            .map(|_| self.broodmother.spawn_test_wave(5))
            .collect();

        let campaign = self.run_waves(waves, SimOpts::default());
        campaign.summary();
        // self.fight_sim(wave);

        self.log_all();
    }

    pub fn run_wave(&mut self, mut wave: Vec<Bug>, opts: SimOpts) -> WaveSummary {
        let clamp = opts.clamp;
        let scale = opts.scale;

        log!(debug, format!("❤️‍🔥 FIGHT START ❤️‍🔥"), true);

        self.rebalance(&mut wave, opts.rebalance_hp, opts.rebalance_ap, opts.rebalance_dmg);
        self.broodmother.debug_wave(&wave);

        let mut round: usize = 1;
        let mut trooper_stats = RollStats::default();
        let mut bug_stats = RollStats::default();

        let mut t_pity_stats = PityStats::default();
        let mut b_pity_stats = PityStats::default();

        let mut t_pity = vec![0u8; self.commander.team.len()];
        let mut b_pity = vec![0u8; wave.len()];

        while self.any_trooper_alive() && Self::any_bug_alive(&wave) {
            log!(info, format!("----- Round {} -----", round), false);

            // --------------------
            // Trooper Phase
            // --------------------
            for ti in 0..self.commander.team.len() {
                if !self.commander.team[ti].is_alive() { continue; }
                let Some(bi) = Self::first_alive_bug_idx(&wave) else { break; };

                // Build once (immutable borrows), then apply damage (mutable) after
                let pity = t_pity[ti];
                let (outcome, final_dmg) = {
                    let atk = Combatant::Trooper(&self.commander.team[ti]);
                    let def = Combatant::Bug(&wave[bi]);

                    let ctx = Self::build_context(atk, def, 0, clamp, pity);
                    let res = Joker::resolve(&mut self.master_rng, &ctx, scale);
                    trooper_stats.record(res.outcome, res.hit_prob_used);
                    t_pity_stats.record(res.base_p, res.hit_prob_used, pity, res.outcome);

                    match res.outcome {
                        HitOutcome::Miss | HitOutcome::Graze => t_pity[ti] = t_pity[ti].saturating_add(1),
                        _ => t_pity[ti] = 0,
                    }

                    (res.outcome, res.final_dmg)
                };

                let (dmg, hp_dmg, ap_dmg) = final_dmg;
                self.broodmother.bug_attacked(&mut wave[bi], dmg, hp_dmg, ap_dmg);

                log!(info, format!(
                    "Trooper#{} -> Bug#{}: {:?}  | dmg={dmg}, hp={hp_dmg}, ap={ap_dmg}",
                    ti + 1, bi + 1, outcome
                ), false);

                if !wave[bi].is_alive() {
                    log!(info, format!("Bug#{} down!", bi + 1), false);
                }
                if !Self::any_bug_alive(&wave) { break; }
            }

            if !Self::any_bug_alive(&wave) { break; }

            // --------------------
            // Bug Phase
            // --------------------
            for bi in 0..wave.len() {
                if !wave[bi].is_alive() { continue; }
                let Some(ti) = self.first_alive_trooper_idx() else { break; };

                let pity = b_pity[bi];
                let (outcome, final_dmg) = {
                    let atk = Combatant::Bug(&wave[bi]);
                    let def = Combatant::Trooper(&self.commander.team[ti]);

                    let ctx = Self::build_context(atk, def, 0, clamp, pity);
                    let res = Joker::resolve(&mut self.master_rng, &ctx, scale);
                    bug_stats.record(res.outcome, res.hit_prob_used);
                    b_pity_stats.record(res.base_p, res.hit_prob_used, pity, res.outcome);

                    match res.outcome {
                        HitOutcome::Miss | HitOutcome::Graze => b_pity[bi] = b_pity[bi].saturating_add(1),
                        _ => b_pity[bi] = 0,
                    }

                    (res.outcome, res.final_dmg)
                };

                let (dmg, hp_dmg, ap_dmg) = final_dmg;
                self.commander.apply_damage_to_trooper(ti, dmg, hp_dmg, ap_dmg);

                log!(info, format!(
                    "Bug#{} -> Trooper#{}: {:?}  | dmg={dmg}, hp={hp_dmg}, ap={ap_dmg}",
                    bi + 1, ti + 1, outcome
                ), false);

                if !self.commander.team[ti].is_alive() {
                    log!(info, format!("Trooper#{} down!", ti + 1), false);
                }
                if !self.any_trooper_alive() { break; }
            }

            round += 1;
            if round > 50 {            // safety cap for runaway fights
                log!(info, "Round cap reached; stopping.", false);
                break;
            }
        }
        
        // Summary
        let alive_t = self.commander.team.iter().filter(|t| t.is_alive()).count();
        let alive_b = wave.iter().filter(|b| b.is_alive()).count();
        log!(info, "===== FIGHT SIM END =====", false);
        log!(info, format!("Survivors — Troopers: {alive_t}, Bugs: {alive_b}"), true);
        log!(info, trooper_stats.summary("Trooper rolls"), false);
        log!(info, bug_stats.summary("Bug rolls"), true);
        log!(info, t_pity_stats.summary("Trooper pity"), false);
        log!(info, b_pity_stats.summary("Bug pity"), true);

        WaveSummary {
            rounds: round.saturating_sub(1),
            trooper_alive: alive_t,
            bug_alive: alive_b,
            trooper_rolls: trooper_stats,
            bug_rolls: bug_stats,
            trooper_pity: t_pity_stats,
            bug_pity: b_pity_stats,
        }
    }

    pub fn between_waves(&mut self) {}

    pub fn run_waves(&mut self, mut waves: Vec<Vec<Bug>>, mut opts: SimOpts) -> CampaignSummary {
        let mut cleared: usize = 0;

        for (wi, mut wave) in waves.drain(..).enumerate() {
            log!(info, format!("🌊 Wave {} begin 🌊", wi + 1), true);

            self.between_waves();

            let enc = self.run_wave(std::mem::take(&mut wave), opts);

            log!(info, format!("🌊 Wave {} end - Rounds: {}, Troopers Alive: {}, Bugs Alive: {}", wi + 1, enc.rounds, enc.trooper_alive, enc.bug_alive), true);

            if enc.trooper_alive == 0 {
                return CampaignSummary { waves_cleared: cleared, last_wave: enc };
            }

            cleared += 1;
        }

        let final_enc = WaveSummary {
            rounds: 0,
            trooper_alive: self.commander.team.iter().filter(|t| t.is_alive()).count(),
            bug_alive: 0,
            trooper_rolls: RollStats::default(),
            bug_rolls: RollStats::default(),
            trooper_pity: PityStats::default(),
            bug_pity: PityStats::default(),
        };

        CampaignSummary { waves_cleared: 0, last_wave: final_enc }
    }

    pub fn fight_sim(&mut self, mut wave: Vec<Bug>) {
        let _enc = self.run_wave(wave, SimOpts::default());
    }

    pub fn log_all(&self) {
        LOG.lock().unwrap().print_all();
    }
}
