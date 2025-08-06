#![allow(dead_code)]
// TODO: Add Chamber Effects, including buffs to troopers/bugs, and debuffs to troopers/bug
// NOTE: At least add a pool or list to choose from, so I can move on to implementing the
// turn_handler
// TODO: Add ChamberState back and implement it
// TODO: Add likely bug types matched to chambertypes, NOT CONNECTED IN CHAMBERSTATE (because
// ChamberState houses which bugs from a wave are alive and their states). Probably actually in
// chamber, which adds a whole refactor nonsense.
// TODO: Rebalance the weights!!
// TODO: Prob remove the imports since I prob won't need them'(Reevaluate after adding effect)
// Imports
use rand::Rng;

use crate::bugs::{Broodmother, Bug};
use crate::troopers::Commander;
use crate::armory::{Armory, Gear};

// Globals

#[derive(Clone, Debug, PartialEq)]
enum ChamberType { Entrance, Tunnel, Clearing, BroodChamber, FoodStorage, Flooded, Collapsed, EggChamber}

#[derive(Clone, Debug)]
enum Hazard {  }

#[derive(Clone, Debug)]
struct ChamberInfo {
    r#type: ChamberType,
    name: &'static str,
    description: &'static str,
    flavor: &'static str,
}

static CHAMBER_INFO: &[ChamberInfo] = &[
    ChamberInfo { 
        r#type: ChamberType::Entrance,
        name: "Entrance",
        description: "The entrance to the hive",
        flavor: "The Opening yaws before you, screeches, chitters, and a faint slithering noise can be heard from inside. The Hive awaits.",
    },
    ChamberInfo { 
        r#type: ChamberType::Tunnel,
        name: "Tunnel",
        description: "The standard connecting passage between chambers. Can contain minor hazards like acid drips or falling debris, and is the most common room type. Sometimes ambushes occur here.",
        flavor: "The tunnel quivers as you move, every echo doubled by the stone. Chitin crunches underfoot. The scent of ammonia lingers in the stale air.",
    },
    ChamberInfo { 
        r#type: ChamberType::Clearing,
        name: "Clearing",
        description: "A rare open space bathed in faint sunlight or bioluminescence. Functions as a safe(ish) room—may heal, regroup, or offer a partial refill.",
        flavor: "You step into a rare pocket of calm where the ceiling has thinned. A shaft of pale light spills down, chasing the shadows and your dread—for a moment, you remember what peace feels like.",
    },
    ChamberInfo { 
        r#type: ChamberType::BroodChamber,
        name: "Brood Chamber",
        description: "High spawn room, “bug nursery.” Walls may pulse with movement. Bug density is increased; expect waves. Sometimes houses a mini-boss.",
        flavor: "The walls are alive with motion. Bulging eggs and writhing larvae line the chamber, and the air vibrates with the faint sound of hatching. A swarm is never far behind.",
    },
    ChamberInfo { 
        r#type: ChamberType::FoodStorage,
        name: "Food Storage",
        description: "Bug larder: decaying corpses, fungal growths, sticky resin. Troopers may heal (if desperate), but risk sickness/debuffs. Common home to Fleshies, Sporebellies.",
        flavor: "Corpses—animal, human, bug—lie stacked and festering, some sealed in resin. Fungi bloom where flesh meets chitin. The stench makes your stomach turn.",
    },
    ChamberInfo { 
        r#type: ChamberType::Flooded,
        name: "Flooded Chamber",
        description: "A partially or fully submerged chamber—slows movement, may damage unprotected troopers, and is favored by Noodles. Some gear or classes may handle it better.",
        flavor: "Cold, dark water laps at your boots as you wade forward. Ripples vanish into the gloom. Something moves beneath the surface, swift and unseen.",
    },
    ChamberInfo { 
        r#type: ChamberType::Collapsed,
        name: "Collapsed Tunnel",
        description: "Passage choked by rubble or cave-ins. Must be cleared to proceed. May risk pinning or ambushes, and can become blocked on retreat.",
        flavor: "A tangle of broken stone and shattered resin blocks the way. Dust chokes the air with every step. The tunnel groans as if it might collapse again at any moment",
    },
    ChamberInfo { 
        r#type: ChamberType::EggChamber,
        name: "Egg Chamber",
        description: "The final room; the Queen’s lair. Unique boss fight. Disturbing eggs may trigger mass spawns or hazards.",
        flavor: "A vast chamber unfolds before you, crowded with throbbing egg sacs and the crawling shapes of new life. The Queen herself towers above all—a living nightmare in flesh and shell.",
    },
];

#[derive(Clone, Debug)]
struct ChamberState {
    enemies: Vec<Bug>,
    is_cleared: bool,
    is_blocked: bool,
    hazards: Vec<Hazard>,
    deployed_gear: Vec<Gear>,
    can_heal_here: bool,
}

#[derive(Clone, Debug)]
struct ChamberWeight {
    r#type: ChamberType,
    weight: u32,
    possible_neighbors: &'static [ChamberType],
}

static CHAMBER_WEIGHTS: &[ChamberWeight] = &[
    ChamberWeight { r#type: ChamberType::Entrance, weight: 0, possible_neighbors: &[ChamberType::Tunnel, ChamberType::Flooded, ChamberType::Collapsed, ChamberType::Clearing] },
    ChamberWeight { r#type: ChamberType::Tunnel, weight: 5, possible_neighbors: &[ChamberType::Tunnel, ChamberType::BroodChamber, ChamberType::Clearing] },
    ChamberWeight { r#type: ChamberType::BroodChamber, weight: 1, possible_neighbors: &[ChamberType::Tunnel, ChamberType::Flooded] },
    ChamberWeight { r#type: ChamberType::Flooded, weight: 2, possible_neighbors: &[ChamberType::Tunnel, ChamberType::BroodChamber] },
    ChamberWeight { r#type: ChamberType::Clearing, weight: 1, possible_neighbors: &[ChamberType::Tunnel, ChamberType::FoodStorage] },
    ChamberWeight { r#type: ChamberType::FoodStorage, weight: 2, possible_neighbors: &[ChamberType::Tunnel, ChamberType::Collapsed] },
    ChamberWeight { r#type: ChamberType::Collapsed, weight: 1, possible_neighbors: &[ChamberType::Tunnel] },
];

static REQ_CHAMBERS: &[(ChamberType, usize, bool)] = &[
    (ChamberType::Clearing, 1, false),
    (ChamberType::Flooded, 1, false),
    (ChamberType::FoodStorage, 1, true),
];

#[derive(Clone, Debug)]
pub struct Chamber {
    id: usize,
    r#type: ChamberType,
    neighbors: Vec<usize>,
    // state: ChamberState,
}

pub struct Cartographer;

impl Cartographer {
    fn restrict_chambers(pool: &mut Vec<ChamberWeight>, ctype: &ChamberType) {
        pool.retain(|cw| &cw.r#type != ctype);
    }

    fn get_chamber_weight<'a>(ctype: &ChamberType, pool: &'a [ChamberWeight]) -> Option<&'a ChamberWeight> {
        pool.iter().find(|cw| &cw.r#type == ctype)
    }

    fn allowed_next_chambers(prev_type: &ChamberType, pool: &[ChamberWeight]) -> Vec<ChamberWeight> {
        if let Some(prev) = Self::get_chamber_weight(prev_type, pool) {
            pool.iter()
                .filter(|cw| prev.possible_neighbors.contains(&cw.r#type))
                .cloned()
                .collect()
        } else {
            pool.to_vec()
        }
    }

    fn weighted_random_type(pool: &[ChamberWeight]) -> ChamberType {
        let total_weight: u32 = pool.iter().map(|cw| cw.weight).sum();
        let mut rng = rand::rng();
        let mut roll = rng.random_range(0..total_weight);
        for cw in pool {
            if roll < cw.weight {
                return cw.r#type.clone();
            }
            roll -= cw.weight;
        }
        pool[0].r#type.clone()
    }

    fn check_validity(chambers: &Vec<ChamberType>, ctype: &ChamberType, pos: usize) -> bool {
        if pos >=2
            && chambers[pos - 1] == *ctype
            && chambers[pos - 2] == *ctype
        {
            return false;
        }

        if (pos == 0 || pos == chambers.len() - 1)
            && REQ_CHAMBERS.iter().any(|(ct, _, _)| ct == ctype)
        {
            return false;
        }

        true
    }

    fn guarantee_chambers(chambers: &mut Vec<ChamberType>, must_have: ChamberType, num: usize) {
        let mut count = chambers.iter().filter(|c| **c == must_have).count();
        let mut rng = rand::rng();
        while count < num {
            let pos = rng.random_range(1..chambers.len() - 1);
            if Self::check_validity(chambers, &must_have, pos) {
                chambers.insert(pos, must_have.clone());
                count += 1;
            }
        }
    }

    fn gen_ctype_list(num_chambers: usize) -> Vec<ChamberType> {
        let mut chambers = Vec::new();
        let mut picker_pool = CHAMBER_WEIGHTS.to_vec();

        chambers.push(ChamberType::Entrance);
        let entrance_neighbors = Self::allowed_next_chambers(&ChamberType::Entrance, &picker_pool);
        let mut ctype = Self::weighted_random_type(&entrance_neighbors);
        chambers.push(ctype);

        for _ in 1..num_chambers {
            let prev_type = chambers.last().unwrap();
            let allowed = Self::allowed_next_chambers(prev_type, &picker_pool);
            let mut tries = 0;
            loop {
                ctype = Self::weighted_random_type(&allowed);
                if Self::check_validity(&chambers, &ctype, chambers.len()) {
                    chambers.push(ctype);
                    break;
                } 
                tries += 1;
                if tries > 20 {
                    chambers.push(ctype);
                    break;
                }
            }
        }
        chambers.push(ChamberType::EggChamber);

        for (ctype, n, restrict) in REQ_CHAMBERS {
            Self::guarantee_chambers(&mut chambers, ctype.clone(), *n);
            if *restrict {
                Self::restrict_chambers(&mut picker_pool, ctype);
            }
        }

        chambers
    }

    fn build_chambers(ctypes: Vec<ChamberType>) -> Vec<Chamber> {
        let mut chambers = Vec::new();
        for (i, ctype) in ctypes.into_iter().enumerate() {
            chambers.push( Chamber { id: i, r#type: ctype, neighbors: vec![]});
        }
        for i in 0..chambers.len() - 1 {
            chambers[i].neighbors.push(i + 1);
            chambers[i + 1].neighbors.push(i);
        }
        chambers
    }

    pub fn print_chambers(chambers: Vec<Chamber>) {
        for chamber in chambers {
            println!("Chamber {} ({:?}) connects to {:?}", chamber.id, chamber.r#type, chamber.neighbors);
        }
    }

    pub fn spawn_chambers(count: usize) {
        let ctypes = Self::gen_ctype_list(count);
        let chambers = Self::build_chambers(ctypes);
        Self::print_chambers(chambers);
    }
}
