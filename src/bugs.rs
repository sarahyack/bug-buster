#![allow(dead_code)]
// TODO: Add way to apply effects to Bug's stats'
// Imports

use rand::{prelude::IndexedRandom};
use std::default::Default;

use crate::{boost, log};
use crate::utils::{SafeSub, RandBools as Bools};

// Enums, Traits, & Constants

#[derive(Debug, Copy, Clone)]
enum BugClass { Charger, Spitter, Swarmer, Hivemind, Pincer, Burrower, Exploder, Jumper, Tank }
#[derive(Debug, Copy, Clone)]
enum BugTactic { Ambush, Rushdown, Flank, Protect, Bait, Adapt, Enrage, Distract, HiveLink }
#[derive(Debug, Copy, Clone)]
enum BugSpecies { Snapper, Maw, Noodle, Priest, Skitter, Leaper, Sporebelly, Fleshcrawler, Blinker, Skulker, Tornaut, Queen }

#[derive(Default, Debug, Copy, Clone)]
struct BugTraits {
    acidic: bool,
    adaptive: bool,
    armored: bool,
    camouflaged: bool,
    explosive: bool,
    hivelink: bool,
    psychic: bool,
    regenerative: bool,
}

#[derive(Default, Debug, Copy, Clone)]
struct BugFlaws {
    acid_leak: bool,
    cracked_shell: bool,
    sickness: bool,
    neural_misfire: bool,
    outcast: bool,
    poor_eyesight: bool,
    sensory_lag: bool,
    sluggish: bool,
}

#[derive(Default, Debug, Copy, Clone)]
struct BugStats {
    hp: u32,
    ap: u32,
    damage: u32,
    accuracy: f32,
    agility: f32,
}

impl BugStats { 
    fn new(hp: u32, ap: u32, damage: u32, accuracy: f32, agility: f32) -> Self {
        BugStats {
            hp,
            ap,
            damage,
            accuracy,
            agility
        }
    } 
}

// Bug Struct

#[derive(Debug, Clone)]
pub struct Bug {
    species: BugSpecies,
    name: &'static str,
    family: &'static str,
    class: BugClass,
    tactic: BugTactic,
    traits: BugTraits,
    flaws: BugFlaws,
    stats: BugStats,
}

impl Bug {
    fn new(species: BugSpecies) -> Self {
        let (class, name, family) = Self::get_species_info(species);

        let tactic = Self::determine_tactic(species);
        let traits = Self::determine_traits(species);
        let flaws = Self::determine_flaws(species);
        let stats = Self::get_stats(species, &traits, &flaws);

        Bug {
            species,
            name,
            family,
            class,
            tactic,
            traits,
            flaws,
            stats,
        }
    }

    fn get_species_info(species: BugSpecies) -> (BugClass, &'static str, &'static str) {
        use BugClass::*;
        use BugSpecies::*;
        match species {
            Snapper => (Charger, "Tharnyx brutus", "Tharnyx"),
            Maw => (Pincer, "Tharnyx dagkitra", "Tharnyx"),
            Noodle => (Spitter, "Varnith ludfliq", "Varnith"),
            Priest => (Hivemind, "Varnith prexis", "Varnith"),
            Skitter => (Swarmer, "Zunari ulithra", "Zunari"),
            Leaper => (Jumper, "Zunari hummari", "Zunari"),
            Sporebelly => (Exploder, "Skolexid mukora", "Skolexid"),
            Fleshcrawler => (Burrower, "Skolexid gorex", "Skolexid"),
            Blinker => (Jumper, "Xethari veletrex", "Xethari"),
            Skulker => (Burrower, "Xethari vitrinar", "Xethari"),
            Tornaut => (Tank, "Tharnyx magna", "Tharnyx"),
            Queen => (Hivemind, "Xethari regina", "Xethari"),
        }
    }

    fn determine_tactic(species: BugSpecies) -> BugTactic {
        use BugSpecies::*;
        use BugTactic::*;
        let options = match species {
            Snapper => vec![Rushdown, Enrage, Flank, Protect],
            Maw => vec![Flank, Ambush, Bait, Distract, HiveLink],
            Noodle => vec![Adapt, Flank, HiveLink, Distract],
            Priest => vec![HiveLink, Distract, Adapt, Bait, Protect],
            Skitter => vec![Rushdown, Bait, Distract, HiveLink],
            Leaper => vec![Ambush, Flank, Enrage, Distract, Adapt],
            Sporebelly => vec![Rushdown, Enrage, Protect, Flank],
            Fleshcrawler => vec![Ambush, Adapt, Bait, HiveLink, Flank],
            Blinker => vec![Protect, Enrage, Ambush, Adapt, Flank],
            Skulker => vec![Ambush, Flank, Bait],
            Tornaut => vec![Rushdown, Enrage, Protect, HiveLink, Distract],
            Queen => vec![Ambush, Rushdown, Flank, Protect, Bait, Adapt, Enrage, Distract, HiveLink],
        };
        *options.choose(&mut rand::rng()).unwrap()
    }

    fn get_species_trait(species: BugSpecies, traits: &mut BugTraits) {
        use BugSpecies::*;
        match species {
            Snapper => traits.armored = true,
            Priest => traits.psychic = true,
            Skitter => traits.hivelink = true,
            Leaper => traits.hivelink = true,
            Sporebelly => traits.explosive = true,
            Blinker => traits.camouflaged = true,
            Tornaut => traits.armored = true,
            _ => {},
        }
    }

    fn get_species_trait_pool(species: BugSpecies, traits: &mut BugTraits) -> Vec<&mut bool> {
        use BugSpecies::*;
        match species {
            Snapper => vec![
                &mut traits.regenerative,
                &mut traits.explosive,
                &mut traits.hivelink,
                &mut traits.acidic,
            ],
            Maw => vec![
                &mut traits.adaptive,
                &mut traits.regenerative,
                &mut traits.camouflaged,
                &mut traits.hivelink,
            ],
            Noodle => vec![
                &mut traits.adaptive,
                &mut traits.psychic,
                &mut traits.armored,
                &mut traits.regenerative,
                &mut traits.camouflaged,
            ],
            Priest => vec![
                &mut traits.armored,
                &mut traits.adaptive,
                &mut traits.acidic,
                &mut traits.camouflaged,
            ],
            Skitter => vec![
                &mut traits.explosive,
                &mut traits.acidic,
                &mut traits.camouflaged,
            ],
            Leaper => vec![
                &mut traits.armored,
                &mut traits.adaptive,
                &mut traits.camouflaged,
                &mut traits.acidic,
                &mut traits.regenerative,
            ],
            Sporebelly => vec![
                &mut traits.armored,
                &mut traits.hivelink,
                &mut traits.acidic,
            ],
            Fleshcrawler => vec![
                &mut traits.camouflaged,
                &mut traits.adaptive,
                &mut traits.acidic,
                &mut traits.explosive,
                &mut traits.psychic,
                &mut traits.armored,
            ],
            Blinker => vec![
                &mut traits.regenerative,
                &mut traits.psychic,
                &mut traits.adaptive,
                &mut traits.acidic,
            ],
            Skulker => vec![
                &mut traits.armored,
                &mut traits.regenerative,
                &mut traits.hivelink,
                &mut traits.adaptive,
            ],
            Tornaut => vec![
                // &mut traits.armored,
                &mut traits.explosive,
                &mut traits.camouflaged,
                &mut traits.hivelink,
                &mut traits.acidic,
            ],
            Queen => vec![
                &mut traits.armored,
                &mut traits.regenerative,
                &mut traits.psychic,
                &mut traits.explosive,
                &mut traits.camouflaged,
                &mut traits.adaptive,
                &mut traits.acidic,
                &mut traits.hivelink,
            ],
        }
    }

    fn determine_traits(species: BugSpecies) -> BugTraits {
        let mut traits = BugTraits { ..Default::default() };
        let mut rng = rand::rng();

        Self::get_species_trait(species, &mut traits);

        let mut trait_pool = Self::get_species_trait_pool(species, &mut traits);

        Bools::roll_bools(&mut trait_pool, &mut rng, 2, 0.5, true);

        traits
    }

    fn get_species_flaw_pool(species: BugSpecies, flaws: &mut BugFlaws) -> Vec<&mut bool> {
        use BugSpecies::*;
        match species {
            Snapper => vec![
                &mut flaws.acid_leak,
                &mut flaws.sensory_lag,
                &mut flaws.outcast,
                &mut flaws.sluggish,
            ],
            Maw => vec![
                &mut flaws.cracked_shell,
                &mut flaws.neural_misfire,
                &mut flaws.sickness,
                &mut flaws.poor_eyesight,
            ],
            Noodle => vec![
                &mut flaws.poor_eyesight,
                &mut flaws.cracked_shell,
                &mut flaws.acid_leak,
                &mut flaws.sensory_lag,
                &mut flaws.neural_misfire,
            ],
            Priest => vec![
                &mut flaws.outcast,
                &mut flaws.sluggish,
                &mut flaws.neural_misfire,
            ],
            Skitter => vec![
                &mut flaws.cracked_shell,
                &mut flaws.acid_leak,
                &mut flaws.sensory_lag,
                &mut flaws.sluggish,
            ],
            Leaper => vec![
                &mut flaws.sluggish,
                &mut flaws.poor_eyesight,
                &mut flaws.neural_misfire,
                &mut flaws.sickness,
            ],
            Sporebelly => vec![
                &mut flaws.acid_leak,
                &mut flaws.cracked_shell,
                &mut flaws.sluggish,
                &mut flaws.neural_misfire,
            ],
            Fleshcrawler => vec![
                &mut flaws.poor_eyesight,
                &mut flaws.outcast,
                &mut flaws.sickness,
                &mut flaws.cracked_shell,
                &mut flaws.acid_leak,
            ],
            Blinker => vec![
                &mut flaws.poor_eyesight,
                &mut flaws.acid_leak,
                &mut flaws.outcast,
                &mut flaws.sickness,
                &mut flaws.neural_misfire,
            ],
            Skulker => vec![
                &mut flaws.acid_leak,
                &mut flaws.sensory_lag,
                &mut flaws.cracked_shell,
            ],
            Tornaut => vec![
                &mut flaws.poor_eyesight,
                &mut flaws.neural_misfire,
                &mut flaws.sluggish,
                &mut flaws.cracked_shell,
            ],
            Queen => vec![
                &mut flaws.sensory_lag,
                &mut flaws.poor_eyesight,
            ],
        }
    }

    fn determine_flaws(species: BugSpecies) -> BugFlaws {
        let mut flaws = BugFlaws { ..Default::default() };
        let mut rng = rand::rng();
        let mut flaw_pool = Self::get_species_flaw_pool(species, &mut flaws);

        Bools::maybe_roll_bools(&mut flaw_pool, &mut rng, 3, 0.5, false, 0.4);

        flaws
    }

    fn get_base_stats(species: BugSpecies) -> BugStats {
        use BugSpecies::*;
        let (hp, ap, damage, accuracy, agility) = match species {
            Snapper => (105, 10, 14, 1.0, 0.4),
            Maw => (170, 32, 25, 1.0, 0.4),
            Noodle => (125, 20, 25, 1.0, 0.7),
            Priest => (150, 40, 15, 1.0, 0.5),
            Skitter => (90, 10, 10, 1.0, 0.7),
            Leaper => (140, 30, 25, 1.0, 0.8),
            Sporebelly => (105, 15, 7, 1.0, 0.2),
            Fleshcrawler => (130, 27, 20, 1.0, 0.6),
            Blinker => (140, 30, 25, 1.0, 0.9),
            Skulker => (130, 27, 20, 1.0, 0.6),
            Tornaut => (250, 50, 30, 1.0, 0.1),
            Queen => (280, 50, 60, 1.0, 0.3),
        };

        BugStats::new(hp, ap, damage, accuracy, agility)
    }

    fn apply_modifiers(stats: &mut BugStats, traits: &BugTraits, flaws: &BugFlaws) -> BugStats {

        boost!(stats, traits.armored, ap += 20);
        boost!(stats, traits.adaptive, agility += 0.1);
        boost!(stats, traits.explosive, damage += 50);
        boost!(stats, traits.explosive, hp -= 10);
        boost!(stats, traits.explosive, ap -= 5);
        boost!(stats, traits.camouflaged, agility = 1.0);
        boost!(stats, traits.camouflaged, hp -= 10);
        boost!(stats, traits.camouflaged, ap -= 10);
        boost!(stats, traits.hivelink, hp += 10);
        boost!(stats, traits.psychic, damage += 10);
        boost!(stats, traits.regenerative, hp += 10);

        boost!(stats, flaws.acid_leak, hp -= 10);
        boost!(stats, flaws.cracked_shell, ap -= 10);
        boost!(stats, flaws.sluggish, agility -= 0.3);
        boost!(stats, flaws.sickness, hp -= 20);
        boost!(stats, flaws.sickness, ap -= 5);
        boost!(stats, flaws.sickness, damage -= 10);
        boost!(stats, flaws.poor_eyesight, accuracy -= 0.3);
        boost!(stats, flaws.poor_eyesight, damage -= 5);
        boost!(stats, flaws.outcast, ap -= 10);

        stats.hp = stats.hp.clamp(10, 300);
        stats.ap = stats.ap.clamp(0, 100);
        stats.damage = stats.damage.clamp(5, 100);
        stats.accuracy = stats.accuracy.clamp(0.1, 1.0);
        stats.agility = stats.agility.clamp(0.1, 1.0);

        stats.accuracy = (stats.accuracy * 100.0).round() / 100.0;
        stats.agility = (stats.agility * 100.0).round() / 100.0;

        *stats
    }

    fn get_stats(species: BugSpecies, traits: &BugTraits, flaws: &BugFlaws) -> BugStats {
        let mut base = Self::get_base_stats(species);
        Self::apply_modifiers(&mut base, traits, flaws)
    }
}

pub struct Broodmother;

impl Broodmother {
    pub fn spawn_test_wave(count: usize) -> Vec<Bug> {
        use BugSpecies::*;
        let species_pool = vec![
            Snapper, Maw, Noodle, Priest, Skitter, Leaper,
            Sporebelly, Fleshcrawler, Blinker, Skulker, Tornaut, Queen,
        ];

        let mut rng = rand::rng();

        (0..count)
            .map(|_| {
                let species = *species_pool.choose(&mut rng).unwrap();
                Bug::new(species)
            })
            .collect()
    }

    pub fn debug_wave(wave: &[Bug]) {
        for (i, bug) in wave.iter().enumerate() {
            log!(info, format!("--- BUG {} ---", i + 1), false);
            log!(info, format!("Species: {:?} ({})", bug.species, bug.name), false);
            log!(info, format!("Family: {}", bug.family), false);
            log!(info, format!("Class: {:?}", bug.class), false);
            log!(info, format!("Tactic: {:?}", bug.tactic), false);
            log!(info, format!("Stats: {:?}", bug.stats), false);
            log!(info, format!("Traits: {:?}", bug.traits), false);
            log!(info, format!("Flaws: {:?}", bug.flaws), true);
        }
    }
}
