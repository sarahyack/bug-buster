#![allow(dead_code)]
// TODO: Create Loadout handling for trooper, possibly being able to pass in Commander?
// TODO: Add way to apply effects to a Trooper's stats'
// TODO: Create way to take damage and way to attack

// ============ Imports =================

use rand::prelude::IndexedRandom;
use std::default::Default;

use crate::{boost, log};
use crate::utils::{SafeSub,RandBools as Bools};
use crate::armory::{Armory, Loadout};
use crate::bugs::{Broodmother, Bug};

// ============ Classes =================

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TrooperClass { Heavy, Scout, Engineer, Medic, ExoTech, Handler, Decoy }
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum ClassPerk { MoraleAura, BugScan, DeployBoost, CombatTriage, ArmorShred, HiveScent, EchoProtocol }

#[derive(Default, Debug, Copy, Clone)]
struct TrooperTraits {
    steadfast: bool,
    adrenal_surge: bool,
    quickdraw: bool,
    hardy: bool,
    mechanic: bool,
    lucky: bool,
    second_wind: bool,
    sharpshooter: bool,
    stubborn: bool,
    battle_medic: bool,
    fast_reflexes: bool,
}

#[derive(Default, Debug, Copy, Clone)]
struct TrooperFlaws {
        old_wounds: bool,
        ammo_glutton: bool,
        jittery: bool,
        acid_phobia: bool,
        nervous_trigger: bool,
        clumsy: bool,
        loudmouth: bool,
        fragile_armor: bool,
        slow_recovery: bool,
        tunnel_vision: bool,
        glass_jaw: bool,
}

#[derive(Default, Debug, Copy, Clone)]
struct TrooperStats {
    hp: u32,
    ap: u32,
    dmg_mod: f32,
    accuracy: f32,
    agility: f32,
}

impl TrooperStats {
    fn new (hp: u32, ap: u32, dmg_mod: f32, accuracy: f32, agility: f32) -> TrooperStats {
        TrooperStats {
            hp,
            ap,
            dmg_mod,
            accuracy,
            agility,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Trooper {
    pub class: TrooperClass,
    loadout: Loadout,
    perk: ClassPerk,
    r#trait: TrooperTraits,
    flaw: TrooperFlaws,
    stats: TrooperStats,
}

impl Trooper {
    fn new(class: TrooperClass) -> Self {
        let loadout = Armory::create_loadout(class);
        let perk = Self::get_class_perk(&class);
        let r#trait = Self::determine_trait();
        let flaw = Self::determine_flaw();
        let stats = Self::get_stats(class, &r#trait, &flaw);

        Trooper {
            class,
            loadout,
            perk,
            r#trait,
            flaw,
            stats,
        }
    }

    fn get_class_perk(class: &TrooperClass) -> ClassPerk {
        match class {
            TrooperClass::Heavy => ClassPerk::MoraleAura,
            TrooperClass::Scout => ClassPerk::BugScan,
            TrooperClass::Engineer => ClassPerk::DeployBoost,
            TrooperClass::Medic => ClassPerk::CombatTriage,
            TrooperClass::ExoTech => ClassPerk::ArmorShred,
            TrooperClass::Handler => ClassPerk::HiveScent,
            TrooperClass::Decoy => ClassPerk::EchoProtocol,
        }
    }

    fn get_trait_pool(traits: &mut TrooperTraits) -> Vec<&mut bool> {
        vec![
            &mut traits.steadfast,
            &mut traits.adrenal_surge,
            &mut traits.quickdraw,
            &mut traits.hardy,
            &mut traits.mechanic,
            &mut traits.lucky,
            &mut traits.second_wind,
            &mut traits.sharpshooter,
            &mut traits.stubborn,
            &mut traits.battle_medic,
            &mut traits.fast_reflexes,
        ]
    }

    fn get_flaw_pool(flaws: &mut TrooperFlaws) -> Vec<&mut bool> {
        vec![
            &mut flaws.old_wounds,
            &mut flaws.ammo_glutton,
            &mut flaws.jittery,
            &mut flaws.acid_phobia,
            &mut flaws.nervous_trigger,
            &mut flaws.clumsy,
            &mut flaws.loudmouth,
            &mut flaws.fragile_armor,
            &mut flaws.slow_recovery,
            &mut flaws.tunnel_vision,
            &mut flaws.glass_jaw,
        ]
    }

    fn determine_trait() -> TrooperTraits {
        let mut traits = TrooperTraits { ..Default::default() };
        let mut rng = rand::rng();

        let mut trait_pool = Self::get_trait_pool(&mut traits);

        Bools::roll_bools(&mut trait_pool, &mut rng, 1,  0.5, true);

        traits
    }

    fn determine_flaw() -> TrooperFlaws {
        let mut flaws = TrooperFlaws { ..Default::default() };
        let mut rng = rand::rng();

        let mut flaw_pool = Self::get_flaw_pool(&mut flaws);

        Bools::roll_bools(&mut flaw_pool, &mut rng, 1, 0.5, true);

        flaws
    }

    fn get_base_stats(class: TrooperClass) -> TrooperStats {
        use TrooperClass::*;
        let (hp, ap, dmg_mod, accuracy, agility) = match class {
            Heavy => (130, 30, 2.0, 0.85, 0.2),
            Scout => (70, 10, 1.25, 1.15, 0.65),
            Engineer => (100, 20, 1.7, 1.05, 0.35),
            Medic => (90, 15, 1.15, 1.05, 0.35),
            ExoTech => (115, 20, 1.9, 0.9, 0.25),
            Handler => (80, 14, 1.3, 1.0, 0.4),
            Decoy => (110, 16, 1.1, 0.8, 0.5),
        };

        TrooperStats::new(hp, ap, dmg_mod, accuracy, agility)
    }

    fn apply_modifiers(stats: &mut TrooperStats, traits: &TrooperTraits, flaws: &TrooperFlaws) -> TrooperStats {
        boost!(stats, traits.sharpshooter, accuracy += 0.15);

        boost!(stats, flaws.old_wounds, hp = ((stats.hp as f32) * 0.85) as u32);

        stats.hp = stats.hp.clamp(10, 200);
        stats.ap = stats.ap.clamp(0, 100);
        stats.dmg_mod = stats.dmg_mod.clamp(0.5, 2.0);
        stats.accuracy = stats.accuracy.clamp(0.5, 1.5);
        stats.agility = stats.agility.clamp(0.0, 1.0);

        stats.dmg_mod = (stats.dmg_mod * 100.0).round() / 100.0;
        stats.accuracy = (stats.accuracy * 100.0).round() / 100.0;
        stats.agility = (stats.agility * 100.0).round() / 100.0; 

        *stats
    }

    fn get_stats(class: TrooperClass, traits: &TrooperTraits, flaws: &TrooperFlaws) -> TrooperStats {
        let mut base = Self::get_base_stats(class);

        Self::apply_modifiers(&mut base, traits, flaws)
    }

    fn damage_mod(&self, base_dmg: u32)  -> u32 {
        let dmg_mod = self.stats.dmg_mod;

        let mod_dmg = (base_dmg as f32 * dmg_mod) as u32;

        mod_dmg
    }

    pub fn hp(&self) -> u32 { self.stats.hp }

    pub fn ap(&self) -> u32 { self.stats.ap }

    pub fn damage(&self) -> (u32, u32, u32) {
        let weapon = self.loadout.equipped_weapon();
        let (dmg, hp_dmg, ap_dmg) = weapon.damage();
        let mod_dmg = self.damage_mod(dmg);
        (
            mod_dmg,
            hp_dmg,
            ap_dmg,
        )
    }

    pub fn accuracy(&self) -> f32 {
        let acc = self.stats.accuracy;
        let equipped_weapon = self.loadout.equipped_weapon();
        let weapon_acc_del = equipped_weapon.accuracy();
        let mult = (1.0 + weapon_acc_del).max(0.5);
        acc * mult
    }

    pub fn agility(&self) -> f32 { self.stats.agility }

    pub fn is_alive(&self) -> bool { self.hp() > 0 }

    pub fn attack(&self, target: &mut Bug) {
        let (dmg, hp_dmg, ap_dmg) = self.damage();
        target.take_damage(dmg, hp_dmg, ap_dmg);
    }

    pub fn take_damage(&mut self, _dmg: u32, hp_dmg: u32, ap_dmg: u32) {
        let stats = &mut self.stats;
        boost!(stats, stats.ap != 0, ap -= ap_dmg);
        boost!(stats, stats.ap == 0, hp -= hp_dmg);
        // boost!(stats, true, hp -= dmg);
        // boost!(stats, true, ap -= dmg);
    }
}

pub struct Commander {
    pub team: Vec<Trooper>,
}

impl Commander {
    pub fn new(count: usize) -> Self {
        let team = Self::test_trooper_creation(count);
        Commander { team }
    }

    pub fn test_trooper_creation(count: usize) -> Vec<Trooper> {
        use TrooperClass::*;
        let class_pool = vec![Heavy, Scout, Engineer, Medic, ExoTech, Handler, Decoy];
        let mut rng = rand::rng();

        (0..count)
            .map(|_| {
                let class = *class_pool.choose(&mut rng).unwrap();
                Trooper::new(class)
            })
            .collect()
    }

    pub fn spawn_troopers(&self, team: &[Trooper]) {
        for (i, trooper) in team.into_iter().enumerate() {
            log!(info, format!(" ======= Trooper {} ======== ", i + 1), false);
            log!(info, format!("Class: {:?}", trooper.class), false);
            log!(info, format!("Perk: {:?}", trooper.perk), false);
            log!(info, format!("Trait: {:?}", trooper.r#trait), false);
            log!(info, format!("Flaw: {:?}", trooper.flaw), false);
            log!(info, format!("Stats: {:?}", trooper.stats), true);
        }
    }

    fn log_trooper_gear(num: usize, trooper: &Trooper) {
        log!(info, format!("//////// Trooper {} \\\\\\\\\\\\\\\\\\\\", num + 1), true);
        log!(info, format!("Trooper Class: {:?}", trooper.class), true);
        Armory::log_loadout(&trooper.loadout);
    }

    pub fn log_team_gear(&self, team: &Vec<Trooper>) {
        for (i, trooper) in team.into_iter().enumerate() {
            Self::log_trooper_gear(i, trooper);
        }
    }

    pub fn trooper_attack(&self, trooper: Trooper, target: &mut Bug) {
        trooper.attack(target);
    }

    pub fn trooper_attacked(&self, trooper: &mut Trooper, dmg: u32, hp_dmg: u32, ap_dmg: u32) {
        trooper.take_damage(dmg, hp_dmg, ap_dmg);
    }

    pub fn apply_damage_to_trooper(&mut self, idx: usize, dmg: u32, hp_dmg: u32, ap_dmg: u32) {
        let target = &mut self.team[idx];
        target.take_damage(dmg, hp_dmg, ap_dmg);
    }

    pub fn rebalance_team(&mut self, hp_factor: f32, ap_factor: f32) {
        for t in &mut self.team {
            t.stats.hp = ((t.stats.hp as f32) * hp_factor).round() as u32;
            t.stats.ap = ((t.stats.ap as f32) * ap_factor).round() as u32;
        }
    }
}
