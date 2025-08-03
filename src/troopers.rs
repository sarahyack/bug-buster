#![allow(dead_code)]

// ============ Imports =================

use rand::prelude::IndexedRandom;
use std::default::Default;

use crate::boost;
use crate::utils::RandBools as Bools;
use crate::armory::Armory;

// ============ Classes =================

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TrooperClass { Heavy, Scout, Engineer, Medic, ExoTech, Handler, Decoy }
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum ClassPerk { MoraleAura, BugScan, DeployBoost, CombatTriage, ArmorShred, HiveScent, EchoProtocol }

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

fn determine_trait() -> TrooperTraits {
    let mut traits = TrooperTraits { ..Default::default() };
    let mut rng = rand::rng();

    let mut trait_pool = get_trait_pool(&mut traits);

    Bools::roll_bools(&mut trait_pool, &mut rng, 1,  0.5, true);

    traits
}

fn determine_flaw() -> TrooperFlaws {
    let mut flaws = TrooperFlaws { ..Default::default() };
    let mut rng = rand::rng();

    let mut flaw_pool = get_flaw_pool(&mut flaws);

    Bools::roll_bools(&mut flaw_pool, &mut rng, 1, 0.5, true);

    flaws
}

fn get_base_stats(class: TrooperClass) -> TrooperStats {
    use TrooperClass::*;
    let (hp, ap, dmg_mod, accuracy, agility) = match class {
        Heavy => (130, 30, 1.2, 0.85, 0.2),
        Scout => (70, 10, 0.95, 1.15, 0.65),
        Engineer => (100, 20, 1.0, 1.05, 0.35),
        Medic => (90, 15, 0.85, 1.05, 0.35),
        ExoTech => (115, 20, 1.3, 0.9, 0.25),
        Handler => (80, 14, 0.9, 1.0, 0.4),
        Decoy => (110, 16, 1.0, 0.8, 0.5),
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
    let mut base = get_base_stats(class);

    apply_modifiers(&mut base, traits, flaws)
}

pub struct Trooper {
    pub class: TrooperClass,
    perk: ClassPerk,
    r#trait: TrooperTraits,
    flaw: TrooperFlaws,
    stats: TrooperStats,
}

impl Trooper {
    fn new(class: TrooperClass) -> Self {
        let perk = get_class_perk(&class);
        let r#trait = determine_trait();
        let flaw = determine_flaw();
        let stats = get_stats(class, &r#trait, &flaw);

        Trooper {
            class,
            perk,
            r#trait,
            flaw,
            stats,
        }
    }
}

pub struct Commander;

impl Commander {
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
    pub fn spawn_troopers(team: &[Trooper]) {
        for (i, trooper) in team.into_iter().enumerate() {
            println!(" ======= Trooper {} ======== ", i + 1);
            println!("Class: {:?}", trooper.class);
            println!("Perk: {:?}", trooper.perk);
            println!("Trait: {:?}", trooper.r#trait);
            println!("Flaw: {:?}", trooper.flaw);
            println!("Stats: {:?}", trooper.stats);
            println!();
        }
    }

    fn print_trooper_gear(num: usize, trooper: Trooper) {
        println!("//////// Trooper {} \\\\\\\\\\\\\\\\\\\\", num + 1);
        Armory::print_class_weapons(trooper.class);
        Armory::print_class_gear(trooper.class);
    }

    pub fn print_team_gear(team: Vec<Trooper>) {
    for (i, trooper) in team.into_iter().enumerate() {
        Self::print_trooper_gear(i, trooper);
    }
    }
}
