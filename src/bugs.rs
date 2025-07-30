#![allow(dead_code)]
// Imports

use rand::{prelude::IndexedRandom, seq::SliceRandom};
use std::default::Default;

use crate::utils::{SafeSub, rand_bool};

// Enums, Traits, & Constants
// TODO: Replace every instance of Debuff/debuff with Flaw. Need to reserve debuff/buff to attack
// effects, since I can't easily think of another name for them.
// TODO: Add Effect/Debuff/Buff system as according to the various kinds of attack effects outlined
// in weapons & gear.

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

// Builder Functions

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

    get_species_trait(species, &mut traits);

    let mut trait_pool = get_species_trait_pool(species, &mut traits);
    let mut assigned = 0;

    trait_pool.shuffle(&mut rng);
    for (_, trait_ref) in trait_pool.into_iter().enumerate() {
        if assigned >= 2 { break; }
        if rand_bool(0.5) || assigned == 0 {
            *trait_ref = true;
            assigned += 1;
        }
    }

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

    if rand_bool(0.4) {
        let mut flaw_pool = get_species_flaw_pool(species, &mut flaws);
        flaw_pool.shuffle(&mut rng);
        for (i, flaw_ref) in flaw_pool.into_iter().enumerate() {
            if i >= 3 { break; }
            if rand_bool(0.5) {
                *flaw_ref = true;
            }
        }
    }

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
    macro_rules! boost {
        ($cond:expr, $field:ident += $val:expr) => {
            if $cond {
                stats.$field += $val;
            }
        };
        ($cond:expr, $field:ident -= $val:expr) => {
            if $cond {
                stats.$field = stats.$field.safe_sub($val);
            }
        };
        ($cond:expr, $field:ident = $expr:expr) => {
            if $cond {
                stats.$field = $expr;
            }
        };
    }

    boost!(traits.armored, ap += 20);
    boost!(traits.adaptive, agility += 0.1);
    boost!(traits.explosive, damage += 50);
    boost!(traits.explosive, hp -= 10);
    boost!(traits.explosive, ap -= 5);
    boost!(traits.camouflaged, agility = 1.0);
    boost!(traits.camouflaged, hp -= 10);
    boost!(traits.camouflaged, ap -= 10);
    boost!(traits.hivelink, hp += 10);
    boost!(traits.psychic, damage += 10);
    boost!(traits.regenerative, hp += 10);

    boost!(flaws.acid_leak, hp -= 10);
    boost!(flaws.cracked_shell, ap -= 10);
    boost!(flaws.sluggish, agility -= 0.3);
    boost!(flaws.sickness, hp -= 20);
    boost!(flaws.sickness, ap -= 5);
    boost!(flaws.sickness, damage -= 10);
    boost!(flaws.poor_eyesight, accuracy -= 0.3);
    boost!(flaws.poor_eyesight, damage -= 5);
    boost!(flaws.outcast, ap -= 10);

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
    let mut base = get_base_stats(species);
    apply_modifiers(&mut base, traits, flaws)
}

// Bug Struct

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
        let (class, name, family) = get_species_info(species);

        let tactic = determine_tactic(species);
        let traits = determine_traits(species);
        let flaws = determine_flaws(species);
        let stats = get_stats(species, &traits, &flaws);

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
}

// TODO: Create Broodmother Struct/methods
// NOTE: The Broodmother struct & it's associated impl & methods are what created the random
// generation of bugs per turn. It's the Spawner essentially.
// NOTE: Additional note, the current implementation is only a test version.

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
            println!("--- BUG {} ---", i + 1);
            println!("Species: {:?} ({})", bug.species, bug.name);
            println!("Family: {}", bug.family);
            println!("Class: {:?}", bug.class);
            println!("Tactic: {:?}", bug.tactic);
            println!("Stats: {:?}", bug.stats);
            println!("Traits: {:?}", bug.traits);
            println!("Flaws: {:?}", bug.flaws);
            println!();
        }
    }
}
