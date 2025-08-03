#![allow(dead_code)]
// Imports
use crate::troopers::TrooperClass;
use rand::prelude::IndexedRandom;

// TODO: Remove Effect Matchup in GearStats (AFTER ARMORY) (See related notes down by GearStats)
// TODO: Create Armory Struct (Handles all weapon/gear creation & logic, but prob for now creates a
// placeholder system for effects, and debug stats generation). The armory should also be able to
// fill Loadout with weapons
// TODO: Create a Loadout Struct either here or in Troopers (prob Troopers), which should determine how
// many slots for weapons and gear the provided trooper has based on the provided class, and then
// is provided with the requested weapons somehow from Armory, and that's where the equipment and
// their effects are stored for use by the turn handler or commander.
// NOTE: The effects for the requested weapons/gear from Loadout should be fetched by Armory
// NOTE: This isn't even taking into account the fact that the weapons are pickable by the player,
// that'll have to be handled by either the commander or maybe the tui? No def the commander. Which
// means I should make the Troopers before I do the Armory.

// GENERAL ARMORY DECLARATIONS

#[derive(Default, Debug, Copy, Clone)]
pub enum Distance { Far, #[default] Normal, Near, Close }
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Usage { Limited(u32), #[default] Unlimited }
enum EquipmentType { Weapon, Gear }
enum EquipmentID {
    WeaponID(WeaponID),
    GearID(GearID),
}

//

// ============ EFFECTS & TARGETING =================

#[derive(Default, Debug, Copy, Clone)]
enum DamageType { #[default] Ballistic, Energy, Explosive, Corrosive, Burn, Physical, Chemical, Repair, Healing }

#[derive(Clone, Debug, Copy)]
pub enum TargetType { Itself, Ally, Enemy, Area, All }
#[derive(Clone, Debug, Copy)]
pub enum Area { Immediate, Neighbors }
#[derive(Clone, Debug, Copy)]
pub enum ReloadPenaltyType { AfterMove, AfterFire, AfterSpecial }

struct TargetStats {
    hp: bool,
    ap: bool,
    accuracy: bool,
    damage: bool,
    agility: bool,
}

#[derive(Default, Copy, Clone, Debug)]
pub enum Effect {
    // Damage & Status
    Stun { turns: u8, area: Option<Area> },         // single or AoE
    Bleed { dmg: u32, turns: u8 },                  // DOT, usually HP
    Burn { dmg: u32, turns: u8, aoe: bool },        // Fire/burn DOT, AoE option
    Poison { dmg: u32, turns: u8, stacks: u8 },     // DOT, stackable
    Corrode { dmg: u32, turns: u8, stacks: u8 },    // DOT, armor-specific
    Knockback { dist: Distance },                        // Move target X tiles
    ChainDamage { dmg: u32, max_targets: u8 },      // Arcs to adjacent enemies
    Cleave { targets: u8 },                         // Hits multiple adjacent
    AoE { dmg: u32, aoe: Area },                   // Explosions, splash, etc.
    Suppress { acc_penalty: i8, turns: u8 },        // Lowers target accuracy
    ArmorPierce { percent: u8 },                    // Ignores some/all AP
    IgnoreArmor,                                    // Direct to HP, ignore AP

    // Buffs & Debuffs
    BuffAP { ap: i32, turns: u8, area: Option<Area> },    // Armor buff, AoE option
    BuffMove { mv: i8, turns: u8 },
    BuffAccuracy { acc: i8, turns: u8 },
    Heal { hp: u32, target: TargetType },                 // Heal self, ally, area
    Regen { hp_per_turn: u32, turns: u8 },                // Ongoing healing
    Revive { hp: u32 },                                   // Bring back from 0
    CleanseDebuffs { target: TargetType },                // Remove debuffs

    // Utility
    #[default] QuickDraw,                                      // Swap as free action
    InfiniteAmmo,                                   // No reload
    RecoverAmmoOnCrit,                              // Recover ammo if crit
    AlwaysSilent,                                   // No aggro
    MarkTarget,                                     // Reveal traits/stats
    RevealTraits,                                   // Full enemy scan
    ActionRefill,                                   // Use again immediately
    Cloak { turns: u8 },                            // Untargetable
    Decoy { duration: u8 },                         // Place decoy to draw aggro
    HoloDouble { turns: u8 },                       // Acts twice, movement penalty after
    Immobilize { turns: u8 },                       // Freeze target
    Trap { duration: u8 },                          // Place trap
    Pacify { turns: u8 },                           // Target can't attack
    Confuse { turns: u8 },                          // Random targeting
    Blind { turns: u8, acc_penalty: i8 },           // Lower accuracy, may restrict attacks
    AggroPull { turns: u8 },                        // Draws enemies to location/user

    // Special
    Custom(&'static str), // fallback for any one-off unique effect
}

#[derive(Default, Copy, Clone, Debug)]
pub enum EquipmentFlaw {
    // Resource/Ammo
    LowAmmo { clip_size: u8 },
    SlowReload { after_shots: u8, turns: u8 },
    Cooldown { turns: u8 },
    CannotCrit,
    PoorAccuracy { penalty: i8 },
    ReloadPenalty { after_action: ReloadPenaltyType },
    OneUsePerRun,
    RequiresSpecificWeaponType(&'static str),

    // Self Harm/Drawbacks
    SelfDamage { dmg: u32, chance: f32 },
    BurnsCover,
    AttractsAggro,
    FriendlyFirePossible,
    StunnedAfterUse { turns: u8 },
    SlowedAfterUse { mv_penalty: i8, turns: u8 },
    AccuracyPenaltyNextTurn { penalty: i8 },
    MovementPenalty { penalty: i8, turns: u8 },
    CantCombineWith(&'static str), // Mutually exclusive gear

    // Limitations
    #[default] OnlyOneActivePerSquad,
    OnlyOnePerTile,
    VisibleToEnemies,
    DestructibleByEnemies { hp: u32 },
    NoEffectIfTargetAtMax,
    NoEffectOnElite,
    OnlyForBeltFed,
    // ...any other specific limitation

    Custom(&'static str), // fallback for one-off/odd-ball flaws
}

//

//  =================== WEAPONS ========================
// Declarations
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum WeaponID { #[default] Minigun, Chaingun, ScopedRifle, PulseSMG, MagShellCannon, VenomSpiker, Flamethrower, SlugCannon, MarksmanCarbine, Railgun, Crossbolt, Spikeshot, AntigenBeam, IonScattergun, AssaultRifle, SMG, RepeaterBow, AutoPistol, SawedOffShotgun, PulsePistol, Spikeling, MicroGrenadeLauncher, HandCannon, Syringer, PlasmaDerringer, BackupRevolver, LightSMG, CombatKnife, PowerMace, ShockBlade, Cleaver, InjectorGauntlet, DoomWrench, AspLash, ArcGauntlet, MonofilamentBlade, TacticalBaton  }

static ALL_WEAPON_IDS: &[WeaponID] = &[
    WeaponID:: Minigun,
    WeaponID:: Chaingun,
    WeaponID:: ScopedRifle,
    WeaponID:: PulseSMG,
    WeaponID:: MagShellCannon,
    WeaponID:: VenomSpiker,
    WeaponID:: Flamethrower,
    WeaponID:: SlugCannon,
    WeaponID:: MarksmanCarbine,
    WeaponID:: Railgun,
    WeaponID:: Crossbolt,
    WeaponID:: Spikeshot,
    WeaponID:: AntigenBeam,
    WeaponID:: IonScattergun,
    WeaponID:: AssaultRifle,
    WeaponID:: SMG,
    WeaponID:: RepeaterBow,
    WeaponID:: AutoPistol,
    WeaponID:: SawedOffShotgun,
    WeaponID:: PulsePistol,
    WeaponID:: Spikeling,
    WeaponID:: MicroGrenadeLauncher,
    WeaponID:: HandCannon,
    WeaponID:: Syringer,
    WeaponID:: PlasmaDerringer,
    WeaponID:: BackupRevolver,
    WeaponID:: LightSMG,
    WeaponID:: CombatKnife,
    WeaponID:: PowerMace,
    WeaponID:: ShockBlade,
    WeaponID:: Cleaver,
    WeaponID:: InjectorGauntlet,
    WeaponID:: DoomWrench,
    WeaponID:: AspLash,
    WeaponID:: ArcGauntlet,
    WeaponID:: MonofilamentBlade,
    WeaponID:: TacticalBaton,
];

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum WeaponType { #[default] Primary, Secondary, Melee }


#[derive(Default, Debug, Copy, Clone)]
struct WeaponInfo {
    id: WeaponID,
    r#type: WeaponType,
    name: &'static str,
    description: &'static str,
    flavor: &'static str,
}


static WEAPON_INFO: &[WeaponInfo] = &[
    // Primary Weapons
    WeaponInfo {
        id:         WeaponID::Minigun,
        r#type:     WeaponType::Primary,
        name:       "Minigun",
        description:"A rapid-fire heavy weapon that unleashes a storm of bullets, suppressing enemies and overwhelming targets at close and medium range. Ideal for holding choke points and cutting down swarms, but requires frequent reloading.",
        flavor:     "Chews through flesh and nerves ... if you can keep it loaded."
    },
    WeaponInfo {
        id:         WeaponID::Chaingun,
        r#type:     WeaponType::Primary,
        name:       "Chaingun",
        description:"A powerful, rotary-barrel machine gun designed to shred through enemy armor and tough exoskeletons. Delivers bursts of high-velocity rounds but kicks hard and slows the user down after sustained fire.",
        flavor:     "Tears armor, tears up your back too."
    },
    WeaponInfo {
        id:         WeaponID::ScopedRifle,
        r#type:     WeaponType::Primary,
        name:       "Scoped Rifle",
        description:"A precision marksman’s rifle with advanced optics for pinpoint targeting. Delivers lethal shots at long range and is perfect for identifying enemy traits and weak points during engagements.",
        flavor:     "One shot, one clean ID."
    },
    WeaponInfo {
        id:         WeaponID::PulseSMG,
        r#type:     WeaponType::Primary,
        name:       "Pulse SMG",
        description:"A lightweight submachine gun that fires energy projectiles in rapid bursts. Its adaptive tech allows for quick suppression of enemies or support of allies, with an integrated healing function in the right hands.",
        flavor:     "Wounds or mends, as the mood strikes."
    },
    WeaponInfo {
        id:         WeaponID::MagShellCannon,
        r#type:     WeaponType::Primary,
        name:       "Mag-Shell Cannon",
        description:"A heavy, single-shot weapon that launches explosive, magnetically-charged shells. Devastates clustered enemies and creates powerful area explosions—built for maximum destruction with minimal subtlety.",
        flavor:     "If it moves, it explodes. If not, it still explodes."
    },
    WeaponInfo {
        id:         WeaponID::VenomSpiker,
        r#type:     WeaponType::Primary,
        name:       "Venom Spiker",
        description:"A compact, rapid-fire launcher that fires toxic, armor-piercing darts. Ideal for applying corrosive effects to enemies over time, especially against heavily shielded or regenerative foes.",
        flavor:     "Dissolves bug shell, then their day."
    },
    WeaponInfo {
        id:         WeaponID::Flamethrower,
        r#type:     WeaponType::Primary,
        name:       "Flamethrower",
        description:"A close-range weapon that projects a sustained jet of fire, ideal for clearing tunnels and burning through swarms. Forces enemies out of cover and applies burning damage in a wide arc.",
        flavor:     "For bugs who don't know when to quit."
    },
    WeaponInfo {
        id:         WeaponID::SlugCannon,
        r#type:     WeaponType::Primary,
        name:       "Slug Cannon",
        description:"A single-shot cannon that fires massive ballistic slugs, sending enemies flying with extreme knockback. Highly effective against large threats, but needs to be reloaded after every shot.",
        flavor:     "Kicks harder than a charging bug."
    },
    WeaponInfo {
        id:         WeaponID::MarksmanCarbine,
        r#type:     WeaponType::Primary,
        name:       "Marksman Carbine",
        description:"A lightweight, semi-automatic rifle designed for high accuracy and quick handling. Especially effective against unaware targets and ideal for rapid repositioning on the battlefield.",
        flavor:     "Fast, light, made for headshots."
    },
    WeaponInfo {
        id:         WeaponID::Railgun,
        r#type:     WeaponType::Primary,
        name:       "Railgun",
        description:"A high-velocity sidearm that uses electromagnetic rails to launch projectiles with immense penetration. Punches through bug armor and is perfect for finishing off stunned or armored threats.",
        flavor:     "Punches through most bug armor."
    },
    WeaponInfo {
        id:         WeaponID::Crossbolt,
        r#type:     WeaponType::Primary,
        name:       "Crossbolt",
        description:"A heavy crossbow variant that launches explosive bolts, which burst into flechettes on impact. Effective for creating area denial zones and hitting clusters of advancing bugs.",
        flavor:     "Every bolt explodes into flechettes."
    },
    WeaponInfo {
        id:         WeaponID::Spikeshot,
        r#type:     WeaponType::Primary,
        name:       "Spikeshot",
        description:"A semi-automatic rifle that fires sharpened projectiles designed to inflict bleeding wounds. Useful for weakening tougher enemies over time and softening up advance waves.",
        flavor:     "Sticks, poisons, and bleeds."
    },
    WeaponInfo {
        id:         WeaponID::AntigenBeam,
        r#type:     WeaponType::Primary,
        name:       "Antigen Beam",
        description:"A precision energy weapon that delivers both offensive and defensive capabilities, damaging enemies while simultaneously healing nearby allies. Designed for medics who need to stay on the move.",
        flavor:     "Heals on the fly—hurts bugs too."
    },
    WeaponInfo {
        id:         WeaponID::IonScattergun,
        r#type:     WeaponType::Primary,
        name:       "Ion Scattergun",
        description:"A high-tech shotgun that fires ionized pellets, arcing energy to nearby targets and disrupting enemy formations. Best for dealing with groups and causing chaos in close quarters.",
        flavor:     "Arcs to more bugs than you’d like to count."
    },
    WeaponInfo {
        id:         WeaponID::AssaultRifle,
        r#type:     WeaponType::Primary,
        name:       "Assault Rifle",
        description:"A standard-issue, all-purpose rifle that balances fire rate, accuracy, and reliability. Effective in any situation and trusted by troopers for its jam-free design.",
        flavor:     "Classic, reliable, no frills."
    },
    WeaponInfo {
        id:         WeaponID::SMG,
        r#type:     WeaponType::Primary,
        name:       "SMG",
        description:"A compact, high-rate-of-fire weapon ideal for close quarters. Excels at putting out a wall of bullets in a pinch, making it a favorite for aggressive or desperate engagements.",
        flavor:     "Spray and pray."
    },
    WeaponInfo {
        id:         WeaponID::RepeaterBow,
        r#type:     WeaponType::Primary,
        name:       "Repeater Bow",
        description:"A versatile bow with a repeating mechanism, capable of firing arrows in quick succession. Suitable for silent eliminations and reusable ammunition when stealth or resourcefulness is needed.",
        flavor:     "Arrows retrieve themselves, mostly."
    },

    // Secondary Weapons
    WeaponInfo {
        id:         WeaponID::AutoPistol,
        r#type:     WeaponType::Secondary,
        name:       "Auto-Pistol",
        description:"A lightweight, semi-automatic sidearm designed for speed and accessibility. Always ready when you need it, making it the ideal backup for any situation where quick reactions matter.",
        flavor:     "Not much stopping power, but it’s always there."
    },
    WeaponInfo {
        id:         WeaponID::SawedOffShotgun,
        r#type:     WeaponType::Secondary,
        name:       "Sawed-Off Shotgun",
        description:"A compact shotgun that delivers a devastating spread at close range. Ideal for last-ditch defense or clearing tight spaces, but limited by its tiny clip and broad scatter.",
        flavor:     "Sometimes subtlety is overrated."
    },
    WeaponInfo {
        id:         WeaponID::PulsePistol,
        r#type:     WeaponType::Secondary,
        name:       "Pulse Pistol",
        description:"A compact energy sidearm that fires bursts of focused pulses, perfect for both self-defense and quick medical support. Can patch up an ally in the heat of battle with a simple swap.",
        flavor:     "A gentle zap to patch, or to prod."
    },
    WeaponInfo {
        id:         WeaponID::Spikeling,
        r#type:     WeaponType::Secondary,
        name:       "Spikeling",
        description:"A small-caliber, toxin-delivering pistol that inflicts poison over time. Its diminutive size belies its ability to wear down even tough opponents with a few well-placed shots.",
        flavor:     "A little prick, a lot of pain."
    },
    WeaponInfo {
        id:         WeaponID::MicroGrenadeLauncher,
        r#type:     WeaponType::Secondary,
        name:       "Micro-Grenade Launcher",
        description:"A handheld launcher designed to fire small explosive rounds around corners or into tight formations. Best used to flush out entrenched enemies or hit clusters in confined spaces.",
        flavor:     "When you must reach around corners."
    },
    WeaponInfo {
        id:         WeaponID::HandCannon,
        r#type:     WeaponType::Secondary,
        name:       "Hand Cannon",
        description:"A high-caliber revolver engineered for sheer stopping power. Unleashes punishing shots that break through armor, though the recoil is nearly as fierce as its bite.",
        flavor:     "Breaks wrists and bugs alike."
    },
    WeaponInfo {
        id:         WeaponID::Syringer,
        r#type:     WeaponType::Secondary,
        name:       "Syringer",
        description:"A precision injector pistol that delivers debilitating chemical payloads. Each shot applies a random debuff, making it a wild card for disrupting enemy plans.",
        flavor:     "Doses targets with random debuffs."
    },
    WeaponInfo {
        id:         WeaponID::PlasmaDerringer,
        r#type:     WeaponType::Secondary,
        name:       "Plasma Derringer",
        description:"A concealable energy pistol that excels at delivering high-voltage plasma rounds, especially effective against stunned or exposed targets. Bypasses cover and packs a sting despite its size.",
        flavor:     "Fits in a gauntlet, stings like a bug."
    },
    WeaponInfo {
        id:         WeaponID::BackupRevolver,
        r#type:     WeaponType::Secondary,
        name:       "Backup Revolver",
        description:"A reliable, old-school sidearm with the ability to double-tap when held steady. Simple, sturdy, and never out of place as a last line of defense.",
        flavor:     "Not much, but sometimes enough."
    },
    WeaponInfo {
        id:         WeaponID::LightSMG,
        r#type:     WeaponType::Secondary,
        name:       "Light SMG",
        description:"A compact submachine gun built for agility and ease of handling. Swaps seamlessly with your primary weapon and lays down a rapid barrage when you need a little extra firepower.",
        flavor:     "For when you can't bring the big one."
    },

    // Melee Weapons
    WeaponInfo {
        id:         WeaponID::CombatKnife,
        r#type:     WeaponType::Melee,
        name:       "Combat Knife",
        description:"A classic close-quarters weapon, perfect for silent takedowns and stealth approaches. Reliable and always ready, it excels when getting up close and personal with no fuss.",
        flavor:     "Never jams. Never runs out."
    },
    WeaponInfo {
        id:         WeaponID::PowerMace,
        r#type:     WeaponType::Melee,
        name:       "Power Mace",
        description:"A heavy, electrified melee weapon designed to crack armor and disrupt enemy nervous systems. Its weighted head delivers concussive force with a jolt, stunning targets on a solid hit.",
        flavor:     "Cracks carapace, fries nerves."
    },
    WeaponInfo {
        id:         WeaponID::ShockBlade,
        r#type:     WeaponType::Melee,
        name:       "Shock Blade",
        description:"An energized sword that slices cleanly through foes and can unleash a chain of electricity to nearby enemies. Especially useful for crowd control and keeping multiple threats at bay.",
        flavor:     "Cuts through bugs, sparks a crowd."
    },
    WeaponInfo {
        id:         WeaponID::Cleaver,
        r#type:     WeaponType::Melee,
        name:       "Cleaver",
        description:"A brutal, oversized axe made for cutting through bug swarms. Its wide arc strikes multiple targets at once, making it perfect for carving a path when surrounded.",
        flavor:     "Loud, mean, and hard to sheath."
    },
    WeaponInfo {
        id:         WeaponID::InjectorGauntlet,
        r#type:     WeaponType::Melee,
        name:       "Injector Gauntlet",
        description:"A punch-activated gauntlet that injects debilitating chemicals into the target. Choose your effect to slow, blind, or weaken enemies—ideal for disrupting high-priority threats.",
        flavor:     "One punch, three options, one outcome."
    },
    WeaponInfo {
        id:         WeaponID::DoomWrench,
        r#type:     WeaponType::Melee,
        name:       "Doom Wrench",
        description:"A heavy-duty tool repurposed for both battlefield repairs and smashing bug carapaces. Repairs allied gear in a pinch or delivers a crushing blow to enemies.",
        flavor:     "Heals gear, hurts bugs."
    },
    WeaponInfo {
        id:         WeaponID::AspLash,
        r#type:     WeaponType::Melee,
        name:       "Asp Lash",
        description:"A flexible, ranged whip laced with venom, striking from a distance and inflicting lingering poison. Keeps you safely out of reach while wearing targets down over time.",
        flavor:     "Hits from afar and keeps on hurting."
    },
    WeaponInfo {
        id:         WeaponID::ArcGauntlet,
        r#type:     WeaponType::Melee,
        name:       "Arc Gauntlet",
        description:"A shock-powered fist weapon that delivers bone-crushing punches and discharges a powerful shockwave on a finishing blow. Built for those who like their melee up close and electrifying.",
        flavor:     "Smashes bugs, shakes the ground."
    },
    WeaponInfo {
        id:         WeaponID::MonofilamentBlade,
        r#type:     WeaponType::Melee,
        name:       "Monofilament Blade",
        description:"A razor-thin, high-tech sword that can cut through armor with ease. Its fragile edge requires care but rewards skilled users with devastatingly quick strikes.",
        flavor:     "Slices bugs before they see you."
    },
    WeaponInfo {
        id:         WeaponID::TacticalBaton,
        r#type:     WeaponType::Melee,
        name:       "Tactical Baton",
        description:"A sturdy, non-lethal melee option designed to stun targets without killing. Dependable and straightforward, it’s the go-to tool for subduing rather than slaying.",
        flavor:     "Tough, basic, reliable."
    },
];

fn get_weapon_info(id: WeaponID) -> WeaponInfo {
    WEAPON_INFO.iter().find(|w|  w.id == id).expect(&format!("Invalid Weapon ID: {:?}", id)).clone()
}

#[derive(Default, Debug, Copy, Clone)]
struct WeaponRestrictions {
    id: WeaponID,
    classes: Option<&'static [TrooperClass]>,
}

static WEAPON_RESTRICTIONS: &[WeaponRestrictions] = &[
    // Primary Weapons
    WeaponRestrictions { id: WeaponID::Minigun, classes: Some(&[TrooperClass::Heavy]) },
    WeaponRestrictions { id: WeaponID::Chaingun, classes: Some(&[TrooperClass::Heavy]) },
    WeaponRestrictions { id: WeaponID::ScopedRifle, classes: Some(&[TrooperClass::Scout]) },
    WeaponRestrictions { id: WeaponID::PulseSMG, classes: Some(&[TrooperClass::Medic]) },
    WeaponRestrictions { id: WeaponID::MagShellCannon, classes: Some(&[TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::VenomSpiker, classes: Some(&[TrooperClass::Handler]) },
    WeaponRestrictions { id: WeaponID::Flamethrower, classes: Some(&[TrooperClass::Heavy, TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::SlugCannon, classes: Some(&[TrooperClass::Heavy]) },
    WeaponRestrictions { id: WeaponID::MarksmanCarbine, classes: Some(&[TrooperClass::Scout]) },
    WeaponRestrictions { id: WeaponID::Railgun, classes: Some(&[TrooperClass::Engineer]) },
    WeaponRestrictions { id: WeaponID::Crossbolt, classes: Some(&[TrooperClass::Engineer]) },
    WeaponRestrictions { id: WeaponID::Spikeshot, classes: Some(&[TrooperClass::Handler]) },
    WeaponRestrictions { id: WeaponID::AntigenBeam, classes: Some(&[TrooperClass::Medic]) },
    WeaponRestrictions { id: WeaponID::IonScattergun, classes: Some(&[TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::AssaultRifle, classes: None },
    WeaponRestrictions { id: WeaponID::SMG, classes: None },
    WeaponRestrictions { id: WeaponID::RepeaterBow, classes: None },

    // Secondary Weapons
    WeaponRestrictions { id: WeaponID::AutoPistol, classes: None},
    WeaponRestrictions { id: WeaponID::SawedOffShotgun, classes: Some(&[TrooperClass::Heavy, TrooperClass::Scout]) },
    WeaponRestrictions { id: WeaponID::PulsePistol, classes: Some(&[TrooperClass::Medic, TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::Spikeling, classes: Some(&[TrooperClass::Handler]) },
    WeaponRestrictions { id: WeaponID::MicroGrenadeLauncher, classes: Some(&[TrooperClass::Heavy, TrooperClass::Engineer]) },
    WeaponRestrictions { id: WeaponID::HandCannon, classes: Some(&[TrooperClass::Heavy]) },
    WeaponRestrictions { id: WeaponID::Syringer, classes: Some(&[TrooperClass::Scout]) },
    WeaponRestrictions { id: WeaponID::PlasmaDerringer, classes: Some(&[TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::BackupRevolver, classes: None },
    WeaponRestrictions { id: WeaponID::LightSMG, classes: None },

    // Melee Weapons
    WeaponRestrictions { id: WeaponID::CombatKnife, classes: None},
    WeaponRestrictions { id: WeaponID::PowerMace, classes: Some(&[TrooperClass::Heavy, TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::ShockBlade, classes: Some(&[TrooperClass::Scout, TrooperClass::Medic]) },
    WeaponRestrictions { id: WeaponID::Cleaver, classes: Some(&[TrooperClass::Heavy, TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::InjectorGauntlet, classes: Some(&[TrooperClass::Handler, TrooperClass::Medic]) },
    WeaponRestrictions { id: WeaponID::DoomWrench, classes: Some(&[TrooperClass::Engineer]) },
    WeaponRestrictions { id: WeaponID::AspLash, classes: Some(&[TrooperClass::Handler]) },
    WeaponRestrictions { id: WeaponID::ArcGauntlet, classes: Some(&[TrooperClass::ExoTech]) },
    WeaponRestrictions { id: WeaponID::MonofilamentBlade, classes: Some(&[TrooperClass::Scout]) },
    WeaponRestrictions { id: WeaponID::TacticalBaton, classes: None },
];

#[derive(Default, Debug, Copy, Clone)]
struct WeaponStats {
    id: WeaponID,
    range: Distance,
    damage_type: &'static [DamageType],
    dmg: u32,
    hp_dmg: u32,
    ap_dmg: u32,
    rof: u32,
    accuracy_delta: f32,
    ammo: Usage,
}

static WEAPON_STATS: &[WeaponStats] = &[
    // ==== Primary Weapons ====
    WeaponStats {
        id:             WeaponID::Minigun,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            10,
        hp_dmg:         38,
        ap_dmg:         34,
        rof:            4,
        accuracy_delta: -0.10,
        ammo:           Usage::Limited(120),
    },
    WeaponStats {
        id:             WeaponID::Chaingun,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            15,
        hp_dmg:         30,
        ap_dmg:         36,
        rof:            2,
        accuracy_delta: -0.05,
        ammo:           Usage::Limited(60),
    },
    WeaponStats {
        id:             WeaponID::ScopedRifle,
        range:          Distance::Far,
        damage_type:    &[DamageType::Ballistic],
        dmg:            16,
        hp_dmg:         32,
        ap_dmg:         29,
        rof:            2,
        accuracy_delta: 0.15,
        ammo:           Usage::Limited(40),
    },
    WeaponStats {
        id:             WeaponID::PulseSMG,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Energy, DamageType::Healing],
        dmg:            8,
        hp_dmg:         24,
        ap_dmg:         21,
        rof:            3,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(80),
    },
    WeaponStats {
        id:             WeaponID::MagShellCannon,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Energy, DamageType::Explosive],
        dmg:            45,
        hp_dmg:         45,
        ap_dmg:         54,
        rof:            1,
        accuracy_delta: -0.20,
        ammo:           Usage::Limited(15),
    },
    WeaponStats {
        id:             WeaponID::VenomSpiker,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Corrosive],
        dmg:            7,
        hp_dmg:         20,
        ap_dmg:         28,
        rof:            3,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(30),
    },
    WeaponStats {
        id:             WeaponID::Flamethrower,
        range:          Distance::Near,
        damage_type:    &[DamageType::Burn],
        dmg:            12,
        hp_dmg:         36,
        ap_dmg:         20,
        rof:            3,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(25),
    },
    WeaponStats {
        id:             WeaponID::SlugCannon,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            48,
        hp_dmg:         48,
        ap_dmg:         40,
        rof:            1,
        accuracy_delta: -0.15,
        ammo:           Usage::Limited(8),
    },
    WeaponStats {
        id:             WeaponID::MarksmanCarbine,
        range:          Distance::Far,
        damage_type:    &[DamageType::Ballistic],
        dmg:            11,
        hp_dmg:         33,
        ap_dmg:         28,
        rof:            3,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(28),
    },
    WeaponStats {
        id:             WeaponID::Railgun,
        range:          Distance::Far,
        damage_type:    &[DamageType::Ballistic, DamageType::Energy],
        dmg:            13,
        hp_dmg:         26,
        ap_dmg:         29,
        rof:            2,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(18),
    },
    WeaponStats {
        id:             WeaponID::Crossbolt,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            18,
        hp_dmg:         18,
        ap_dmg:         25,
        rof:            1,
        accuracy_delta: 0.0,
        ammo:           Usage::Limited(12),
    },
    WeaponStats {
        id:             WeaponID::Spikeshot,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            12,
        hp_dmg:         24,
        ap_dmg:         30,
        rof:            2,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(30),
    },
    WeaponStats {
        id:             WeaponID::AntigenBeam,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Energy, DamageType::Healing],
        dmg:            10,
        hp_dmg:         20,
        ap_dmg:         12,
        rof:            2,
        accuracy_delta: 0.20,
        ammo:           Usage::Limited(20),
    },
    WeaponStats {
        id:             WeaponID::IonScattergun,
        range:          Distance::Near,
        damage_type:    &[DamageType::Energy],
        dmg:            18,
        hp_dmg:         36,
        ap_dmg:         24,
        rof:            2,
        accuracy_delta: -0.10,
        ammo:           Usage::Limited(20),
    },
    WeaponStats {
        id:             WeaponID::AssaultRifle,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            9,
        hp_dmg:         27,
        ap_dmg:         24,
        rof:            3,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(35),
    },
    WeaponStats {
        id:             WeaponID::SMG,
        range:          Distance::Normal,
        damage_type:    &[DamageType::Ballistic],
        dmg:            7,
        hp_dmg:         28,
        ap_dmg:         18,
        rof:            4,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(40),
    },
    WeaponStats {
        id:             WeaponID::RepeaterBow,
        range:          Distance::Far,
        damage_type:    &[DamageType::Physical],
        dmg:            15,
        hp_dmg:         30,
        ap_dmg:         28,
        rof:            2,
        accuracy_delta: 0.0,
        ammo:           Usage::Limited(16),
    },

    // ==== Secondary Weapons ====
    WeaponStats {
        id:             WeaponID::AutoPistol,
        range:          Distance::Near,
        damage_type:    &[DamageType::Ballistic],
        dmg:            6,
        hp_dmg:         12,
        ap_dmg:         10,
        rof:            2,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(18),
    },
    WeaponStats {
        id:             WeaponID::SawedOffShotgun,
        range:          Distance::Near,
        damage_type:    &[DamageType::Ballistic],
        dmg:            13,
        hp_dmg:         13,
        ap_dmg:         11,
        rof:            1,
        accuracy_delta: -0.05,
        ammo:           Usage::Limited(6),
    },
    WeaponStats {
        id:             WeaponID::PulsePistol,
        range:          Distance::Near,
        damage_type:    &[DamageType::Energy, DamageType::Healing],
        dmg:            7,
        hp_dmg:         14,
        ap_dmg:         12,
        rof:            2,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(16),
    },
    WeaponStats {
        id:             WeaponID::Spikeling,
        range:          Distance::Near,
        damage_type:    &[DamageType::Chemical],
        dmg:            8,
        hp_dmg:         16,
        ap_dmg:         13,
        rof:            2,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(10),
    },
    WeaponStats {
        id:             WeaponID::MicroGrenadeLauncher,
        range:          Distance::Near,
        damage_type:    &[DamageType::Explosive],
        dmg:            20,
        hp_dmg:         20,
        ap_dmg:         17,
        rof:            1,
        accuracy_delta: -0.10,
        ammo:           Usage::Limited(4),
    },
    WeaponStats {
        id:             WeaponID::HandCannon,
        range:          Distance::Near,
        damage_type:    &[DamageType::Ballistic],
        dmg:            18,
        hp_dmg:         18,
        ap_dmg:         22,
        rof:            1,
        accuracy_delta: -0.15,
        ammo:           Usage::Limited(6),
    },
    WeaponStats {
        id:             WeaponID::Syringer,
        range:          Distance::Near,
        damage_type:    &[DamageType::Chemical],
        dmg:            8,
        hp_dmg:         16,
        ap_dmg:         11,
        rof:            2,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(12),
    },
    WeaponStats {
        id:             WeaponID::PlasmaDerringer,
        range:          Distance::Near,
        damage_type:    &[DamageType::Energy],
        dmg:            14,
        hp_dmg:         14,
        ap_dmg:         10,
        rof:            1,
        accuracy_delta: 0.05,
        ammo:           Usage::Limited(8),
    },
    WeaponStats {
        id:             WeaponID::BackupRevolver,
        range:          Distance::Near,
        damage_type:    &[DamageType::Ballistic],
        dmg:            10,
        hp_dmg:         10,
        ap_dmg:         9,
        rof:            1,
        accuracy_delta: 0.10,
        ammo:           Usage::Limited(8),
    },
    WeaponStats {
        id:             WeaponID::LightSMG,
        range:          Distance::Near,
        damage_type:    &[DamageType::Ballistic],
        dmg:            7,
        hp_dmg:         21,
        ap_dmg:         14,
        rof:            3,
        accuracy_delta: 0.15,
        ammo:           Usage::Limited(16),
    },

    // ==== Melee Weapons ====
    WeaponStats {
        id:             WeaponID::CombatKnife,
        range:          Distance::Close,
        damage_type:    &[DamageType::Physical],
        dmg:            22,
        hp_dmg:         22,
        ap_dmg:         26,
        rof:            1,
        accuracy_delta: 0.05,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::PowerMace,
        range:          Distance::Close,
        damage_type:    &[DamageType::Energy, DamageType::Physical],
        dmg:            35,
        hp_dmg:         35,
        ap_dmg:         42,
        rof:            1,
        accuracy_delta: -0.10,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::ShockBlade,
        range:          Distance::Close,
        damage_type:    &[DamageType::Energy],
        dmg:            24,
        hp_dmg:         24,
        ap_dmg:         30,
        rof:            1,
        accuracy_delta: 0.00,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::Cleaver,
        range:          Distance::Close,
        damage_type:    &[DamageType::Physical],
        dmg:            32,
        hp_dmg:         32,
        ap_dmg:         38,
        rof:            1,
        accuracy_delta: -0.05,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::InjectorGauntlet,
        range:          Distance::Close,
        damage_type:    &[DamageType::Chemical],
        dmg:            18,
        hp_dmg:         18,
        ap_dmg:         22,
        rof:            1,
        accuracy_delta: 0.05,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::DoomWrench,
        range:          Distance::Close,
        damage_type:    &[DamageType::Physical, DamageType::Repair],
        dmg:            26,
        hp_dmg:         26,
        ap_dmg:         32,
        rof:            1,
        accuracy_delta: 0.00,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::AspLash,
        range:          Distance::Close, // If you want to add Distance::Reach, just update this here
        damage_type:    &[DamageType::Physical, DamageType::Chemical],
        dmg:            16,
        hp_dmg:         16,
        ap_dmg:         20,
        rof:            1,
        accuracy_delta: 0.05,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::ArcGauntlet,
        range:          Distance::Close,
        damage_type:    &[DamageType::Energy],
        dmg:            30,
        hp_dmg:         30,
        ap_dmg:         34,
        rof:            1,
        accuracy_delta: -0.05,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::MonofilamentBlade,
        range:          Distance::Close,
        damage_type:    &[DamageType::Energy],
        dmg:            27,
        hp_dmg:         27,
        ap_dmg:         32,
        rof:            1,
        accuracy_delta: 0.10,
        ammo:           Usage::Unlimited,
    },
    WeaponStats {
        id:             WeaponID::TacticalBaton,
        range:          Distance::Close,
        damage_type:    &[DamageType::Physical],
        dmg:            18,
        hp_dmg:         18,
        ap_dmg:         19,
        rof:            1,
        accuracy_delta: 0.00,
        ammo:           Usage::Unlimited,
    },
];

fn get_weapon_stats(id: WeaponID) -> WeaponStats {
    WEAPON_STATS.iter().find(|w|  w.id == id).expect(&format!("Invalid Weapon ID: {:?}", id)).clone()
}

#[derive(Default, Clone, Debug)]
pub struct Weapon {
    id: WeaponID,
    info: WeaponInfo,
    stats: WeaponStats,
    effect: Option<Effect>,
    flaw: Option<EquipmentFlaw>,
}

impl Weapon {
    fn new(id: WeaponID) -> Self {
        let info = get_weapon_info(id);
        let stats = get_weapon_stats(id);
        
        Weapon {
            id,
            info,
            stats,
            effect: None,
            flaw: None,
        }
    }
}

//

// ====================== GEAR ===========================
// TODO: Create GearStats, get_gear_stats(), and then can implement get_equipment_stats() which
// will serve as the actual fetcher function for both structs. Possibly, either that or it'll only
// be used as a external sort of api function, idk.

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum GearID { #[default] ReinforcedPlating, AmmoFeederRig, BlastShield, ShockwaveGrenade, CloakFieldUnit, GrappleLauncher, EchoBeacon, HoloDecoy, NanoMistInjector, StabilizerDrone, AntitoxinSpray, Painkillers, AutoTurret, PatchKit, SensorNode, LaserTripwire, PortableMinefield, ArcWelder, DetonationRemote, NanoGlueBomb, PlasmaCutter, HiveScanner, ChitinBait, ConfusionCollar, BugPheromoneBomb, ShellPack, PlasmaShield, UltraShredRounds, GravityField, EchoPulse, HoloDoubler, DoppelgangerSuit, NanoPatch, StimPack, FragGrenade, SmokeBomb, AdrenalineInjector, TrapKit }

static ALL_GEAR_IDS: &[GearID] = &[
    GearID::ReinforcedPlating,
    GearID::AmmoFeederRig,
    GearID::BlastShield,
    GearID::ShockwaveGrenade,
    GearID::CloakFieldUnit,
    GearID::GrappleLauncher,
    GearID::EchoBeacon,
    GearID::HoloDecoy,
    GearID::NanoMistInjector,
    GearID::StabilizerDrone,
    GearID::AntitoxinSpray,
    GearID::Painkillers,
    GearID::AutoTurret,
    GearID::PatchKit,
    GearID::SensorNode,
    GearID::LaserTripwire,
    GearID::PortableMinefield,
    GearID::ArcWelder,
    GearID::DetonationRemote,
    GearID::NanoGlueBomb,
    GearID::PlasmaCutter,
    GearID::HiveScanner,
    GearID::ChitinBait,
    GearID::ConfusionCollar,
    GearID::BugPheromoneBomb,
    GearID::ShellPack,
    GearID::PlasmaShield,
    GearID::UltraShredRounds,
    GearID::GravityField,
    GearID::EchoPulse,
    GearID::HoloDoubler,
    GearID::DoppelgangerSuit,
    GearID::NanoPatch,
    GearID::StimPack,
    GearID::FragGrenade,
    GearID::SmokeBomb,
    GearID::AdrenalineInjector,
    GearID::TrapKit,
];

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum GearType {
    #[default]
    Wearable,
    Utility,
    Deployable,
    Consumable,
    Throwable,
    UtilityMelee,
}

#[derive(Default, Debug, Copy, Clone)]
struct GearInfo {
    pub id: GearID,
    pub r#type: GearType,
    pub name: &'static str,
    pub description: &'static str,
    pub flavor: &'static str,
}

static GEAR_INFO: &[GearInfo] = &[
    GearInfo {
        id:             GearID::ReinforcedPlating,
        r#type:         GearType::Wearable,
        name:           "Reinforced Plating",
        description:    "Heavy-duty armor plates designed to absorb and deflect bug attacks. Ideal for troopers who need to take the front line and soak up punishment, trading speed for unmatched durability.",
        flavor:         "You're a walking tank—just not a running one.",
    },
    GearInfo {
        id:             GearID::AmmoFeederRig,
        r#type:         GearType::Utility,
        name:           "Ammo Feeder Rig",
        description:    "A belt-fed reloading system that instantly supplies fresh ammo to your primary weapon. Perfect for heavy gunners who need to keep firing without pausing to reload.",
        flavor:         "When the horde keeps coming, so do you.",
    },
    GearInfo {
        id:             GearID::BlastShield,
        r#type:         GearType::Deployable,
        name:           "Blast Shield",
        description:    "A deployable shield that provides solid cover and absorbs heavy fire from one direction. Essential for holding choke points or giving the squad a moment to regroup and reload.",
        flavor:         "Duck behind, breathe easy, reload.",
    },
    GearInfo {
        id:             GearID::ShockwaveGrenade,
        r#type:         GearType::Throwable,
        name:           "Shockwave Grenade",
        description:    "A powerful stun grenade that unleashes a concussive blast in a wide cone, incapacitating nearby enemies and drawing their attention for follow-up attacks.",
        flavor:         "When you want the bugs’ attention—and then a nap.",
    },
    GearInfo {
        id:             GearID::CloakFieldUnit,
        r#type:         GearType::Utility,
        name:           "Cloak Field Unit",
        description:    "Personal cloaking tech that renders the user untargetable for a brief window. Best used for repositioning or escaping tight spots, but deactivates if you go on the attack.",
        flavor:         "The best bug is a confused bug.",
    },
    GearInfo {
        id:             GearID::GrappleLauncher,
        r#type:         GearType::Utility,
        name:           "Grapple Launcher",
        description:    "A compact launcher that fires a grappling hook, letting you traverse the battlefield quickly and bypass obstacles. Excellent for vertical movement or rapid repositioning.",
        flavor:         "Think vertically—bugs can't.",
    },
    GearInfo {
        id:             GearID::EchoBeacon,
        r#type:         GearType::Deployable,
        name:           "Echo Beacon",
        description:    "A deployable device that emits a signal to lure nearby bugs to its location, diverting enemy attention and opening up tactical options for the squad.",
        flavor:         "For when you need bugs to look the other way.",
    },
    GearInfo {
        id:             GearID::HoloDecoy,
        r#type:         GearType::Deployable,
        name:           "Holo Decoy",
        description:    "A stationary holographic projection that distracts and draws aggro from enemies for a short period. Useful for buying time or setting up an ambush.",
        flavor:         "Sometimes a fake is all you need.",
    },
    GearInfo {
        id:             GearID::NanoMistInjector,
        r#type:         GearType::Consumable,
        name:           "Nano-Mist Injector",
        description:    "A medical device that releases healing nanobots in a fine mist, rapidly restoring health and removing debuffs from nearby allies in a pinch.",
        flavor:         "Breath deep, walk it off.",
    },
    GearInfo {
        id:             GearID::StabilizerDrone,
        r#type:         GearType::Deployable,
        name:           "Stabilizer Drone",
        description:    "A portable medical drone that automatically revives and heals downed troopers over multiple turns. Keeps your squad in the fight when you’re out of reach.",
        flavor:         "Trust the drone more than your squad.",
    },
    GearInfo {
        id:             GearID::AntitoxinSpray,
        r#type:         GearType::Consumable,
        name:           "Antitoxin Spray",
        description:    "A compact aerosol that neutralizes poison and corrosive effects instantly on a single ally, providing critical support in hazardous environments.",
        flavor:         "That burning feeling is just health returning.",
    },
    GearInfo {
        id:             GearID::Painkillers,
        r#type:         GearType::Consumable,
        name:           "Painkillers",
        description:    "Fast-acting injectors that remove the effects of being stunned from an ally, though at a health cost after the effect wears off. Best used in emergencies.",
        flavor:         "Not for recreational use... technically.",
    },
    GearInfo {
        id:             GearID::AutoTurret,
        r#type:         GearType::Deployable,
        name:           "Auto-Turret",
        description:    "A deployable sentry that automatically targets and fires at approaching enemies, providing automated fire support and holding ground against waves.",
        flavor:         "Set it, forget it, let it work.",
    },
    GearInfo {
        id:             GearID::PatchKit,
        r#type:         GearType::Consumable,
        name:           "Patch Kit",
        description:    "A portable repair kit that restores armor points to troopers on the fly, extending their survivability in prolonged engagements.",
        flavor:         "Makes duct tape obsolete.",
    },
    GearInfo {
        id:             GearID::SensorNode,
        r#type:         GearType::Deployable,
        name:           "Sensor Node",
        description:    "A one-use electronic sensor that scans the area and reveals hidden bugs within a wide radius, eliminating ambush threats and improving tactical awareness.",
        flavor:         "There’s nowhere to hide now.",
    },
    GearInfo {
        id:             GearID::LaserTripwire,
        r#type:         GearType::Deployable,
        name:           "Laser Tripwire",
        description:    "A deployable trap that emits a visible beam, stunning and damaging the first bug to cross its path. Especially effective in chokepoints and corridors.",
        flavor:         "Bugs see the light—too late.",
    },
    GearInfo {
        id:             GearID::PortableMinefield,
        r#type:         GearType::Deployable,
        name:           "Portable Minefield",
        description:    "A set of mines deployed to cover a zone, dealing heavy damage to any enemy that enters. Careful placement is required to avoid friendly casualties.",
        flavor:         "Dance if you dare.",
    },
    GearInfo {
        id:             GearID::ArcWelder,
        r#type:         GearType::Utility,
        name:           "Arc Welder",
        description:    "A multi-use tool capable of repairing trooper armor or discharging a damaging arc of electricity to adjacent enemies. Versatile for both defense and offense.",
        flavor:         "Weld or wound, your choice.",
    },
    GearInfo {
        id:             GearID::DetonationRemote,
        r#type:         GearType::Utility,
        name:           "Detonation Remote",
        description:    "A handheld device for remote activation of all deployed mines and turrets, allowing for coordinated traps and surprise attacks on the enemy.",
        flavor:         "Detonate everything—sometimes including the plan.",
    },
    GearInfo {
        id:             GearID::NanoGlueBomb,
        r#type:         GearType::Throwable,
        name:           "Nano-Glue Bomb",
        description:    "A thrown adhesive device that immobilizes a bug for one turn and slows them afterward, providing control over high-priority targets.",
        flavor:         "Stick around and suffer.",
    },
    GearInfo {
        id:             GearID::PlasmaCutter,
        r#type:         GearType::UtilityMelee,
        name:           "Plasma Cutter",
        description:    "A compact, high-energy cutting tool that breaches obstacles or deals focused energy damage to adjacent targets. Perfect for engineering solutions or close-quarters emergencies.",
        flavor:         "When subtlety isn’t an option.",
    },
    GearInfo {
        id:             GearID::HiveScanner,
        r#type:         GearType::Utility,
        name:           "Hive Scanner",
        description:    "A scanning device that reveals all stats and traits of a target bug, giving your squad valuable intel for tactical decision-making.",
        flavor:         "Knows more about bugs than they do.",
    },
    GearInfo {
        id:             GearID::ChitinBait,
        r#type:         GearType::Consumable,
        name:           "Chitin Bait",
        description:    "A chemical lure made from processed bug chitin that pacifies a target for a short duration, preventing them from attacking and buying precious time.",
        flavor:         "The secret ingredient is always more bug.",
    },
    GearInfo {
        id:             GearID::ConfusionCollar,
        r#type:         GearType::Utility,
        name:           "Confusion Collar",
        description:    "A wearable device that scrambles a bug’s targeting systems, causing them to attack randomly and lose focus for a turn.",
        flavor:         "Bugs forget who they're mad at.",
    },
    GearInfo {
        id:             GearID::BugPheromoneBomb,
        r#type:         GearType::Throwable,
        name:           "Bug Pheromone Bomb",
        description:    "A powerful pheromone dispersal device that causes all bugs in range to attack the nearest target, sowing chaos among enemy ranks.",
        flavor:         "All's fair in bug love and war.",
    },
    GearInfo {
        id:             GearID::ShellPack,
        r#type:         GearType::Consumable,
        name:           "Shell Pack",
        description:    "An ammo pack that replenishes a large amount of ammunition to your current weapon, ensuring you never run dry when the fighting gets heavy.",
        flavor:         "Big gun, big hunger.",
    },
    GearInfo {
        id:             GearID::PlasmaShield,
        r#type:         GearType::Deployable,
        name:           "Plasma Shield",
        description:    "A deployable energy field that boosts the armor of nearby allies for a short duration, providing a critical defensive advantage in tight situations.",
        flavor:         "A bubble of peace in a hive of chaos.",
    },
    GearInfo {
        id:             GearID::UltraShredRounds,
        r#type:         GearType::Consumable,
        name:           "Ultra-Shred Rounds",
        description:    "Specialized ammunition that lets all Exo-Tech attacks bypass armor entirely for a short time, dealing damage directly to enemy health.",
        flavor:         "Armor? What armor?",
    },
    GearInfo {
        id:             GearID::GravityField,
        r#type:         GearType::Deployable,
        name:           "Gravity Field",
        description:    "A deployable device that generates a heavy gravitational pull, slowing all bugs within its radius and controlling the flow of battle.",
        flavor:         "Now the bugs crawl like you want them to.",
    },
    GearInfo {
        id:             GearID::EchoPulse,
        r#type:         GearType::Utility,
        name:           "Echo Pulse",
        description:    "A pulse generator that temporarily disrupts enemy coordination, causing all bugs to lose one action on their next turn.",
        flavor:         "Stuns more than just bugs.",
    },
    GearInfo {
        id:             GearID::HoloDoubler,
        r#type:         GearType::Utility,
        name:           "Holo-Doubler",
        description:    "A wearable utility that allows the user to act twice in a single turn, at the cost of movement speed afterward. Perfect for high-risk, high-reward plays.",
        flavor:         "Twice the action, half the coordination.",
    },
    GearInfo {
        id:             GearID::DoppelgangerSuit,
        r#type:         GearType::Wearable,
        name:           "Doppelganger Suit",
        description:    "A wearable suit that temporarily copies another trooper’s class and gear, letting you adapt to battlefield needs while drawing enemy attention.",
        flavor:         "Twice the decoy, twice the fun.",
    },
    GearInfo {
        id:             GearID::NanoPatch,
        r#type:         GearType::Wearable,
        name:           "Nano-Patch",
        description:    "A wearable patch that attaches to a trooper, granting automatic health regeneration for the duration of the mission. Only one can be active per squad at a time.",
        flavor:         "You’ll barely notice the bots—until you miss them.",
    },
    GearInfo {
        id:             GearID::StimPack,
        r#type:         GearType::Consumable,
        name:           "Stim Pack",
        description:    "A combat stim that temporarily boosts movement speed and grants immunity to debuffs, with a health penalty once the effect fades.",
        flavor:         "Run first, recover later.",
    },
    GearInfo {
        id:             GearID::FragGrenade,
        r#type:         GearType::Throwable,
        name:           "Frag Grenade",
        description:    "A standard explosive grenade that deals heavy damage in a small radius and ignores armor. Effective for clearing clusters and tough enemies.",
        flavor:         "The universal solution.",
    },
    GearInfo {
        id:             GearID::SmokeBomb,
        r#type:         GearType::Throwable,
        name:           "Smoke Bomb",
        description:    "A tactical grenade that shrouds allies in smoke, making them untargetable for a turn but also reducing visibility for both sides.",
        flavor:         "Now you see us, now you don’t.",
    },
    GearInfo {
        id:             GearID::AdrenalineInjector,
        r#type:         GearType::Consumable,
        name:           "Adrenaline Injector",
        description:    "A lifesaving auto-injector that revives a downed trooper and restores a portion of their health, at the cost of a brief period of vulnerability.",
        flavor:         "Death is just an inconvenience.",
    },
    GearInfo {
        id:             GearID::TrapKit,
        r#type:         GearType::Deployable,
        name:           "Trap Kit",
        description:    "A deployable trap that immobilizes the first bug to enter its tile, providing crowd control and area denial when carefully placed.",
        flavor:         "It’s not paranoia if the bugs really are everywhere.",
    },
];

fn get_gear_info(id: GearID) -> GearInfo {
    GEAR_INFO.iter().find(|g| g.id == id).expect(&format!("Invalid Gear ID: {:?}", id)).clone()
}

#[derive(Default, Debug, Copy, Clone)]
struct GearRestrictions {
    id: GearID,
    classes: Option<&'static [TrooperClass]>,
}

static GEAR_RESTRICTIONS: &[GearRestrictions] = &[
    GearRestrictions { id: GearID::ReinforcedPlating,    classes: Some(&[TrooperClass::Heavy]) },
    GearRestrictions { id: GearID::AmmoFeederRig,        classes: Some(&[TrooperClass::Heavy]) },
    GearRestrictions { id: GearID::BlastShield,          classes: Some(&[TrooperClass::Heavy]) },
    GearRestrictions { id: GearID::ShockwaveGrenade,     classes: Some(&[TrooperClass::Heavy]) },

    GearRestrictions { id: GearID::CloakFieldUnit,       classes: Some(&[TrooperClass::Scout]) },
    GearRestrictions { id: GearID::GrappleLauncher,      classes: Some(&[TrooperClass::Scout]) },
    GearRestrictions { id: GearID::EchoBeacon,           classes: Some(&[TrooperClass::Scout]) },
    GearRestrictions { id: GearID::HoloDecoy,            classes: Some(&[TrooperClass::Scout, TrooperClass::Decoy]) },

    GearRestrictions { id: GearID::NanoMistInjector,     classes: Some(&[TrooperClass::Medic]) },
    GearRestrictions { id: GearID::StabilizerDrone,      classes: Some(&[TrooperClass::Medic]) },
    GearRestrictions { id: GearID::AntitoxinSpray,       classes: Some(&[TrooperClass::Medic]) },
    GearRestrictions { id: GearID::Painkillers,          classes: Some(&[TrooperClass::Medic]) },

    GearRestrictions { id: GearID::AutoTurret,           classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::PatchKit,             classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::SensorNode,           classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::LaserTripwire,        classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::PortableMinefield,    classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::ArcWelder,            classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::DetonationRemote,     classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::NanoGlueBomb,         classes: Some(&[TrooperClass::Engineer]) },
    GearRestrictions { id: GearID::PlasmaCutter,         classes: Some(&[TrooperClass::Engineer]) },

    GearRestrictions { id: GearID::HiveScanner,          classes: Some(&[TrooperClass::Handler]) },
    GearRestrictions { id: GearID::ChitinBait,           classes: Some(&[TrooperClass::Handler]) },
    GearRestrictions { id: GearID::ConfusionCollar,      classes: Some(&[TrooperClass::Handler]) },
    GearRestrictions { id: GearID::BugPheromoneBomb,     classes: Some(&[TrooperClass::Handler]) },

    GearRestrictions { id: GearID::ShellPack,            classes: Some(&[TrooperClass::ExoTech]) },
    GearRestrictions { id: GearID::PlasmaShield,         classes: Some(&[TrooperClass::ExoTech]) },
    GearRestrictions { id: GearID::UltraShredRounds,     classes: Some(&[TrooperClass::ExoTech]) },
    GearRestrictions { id: GearID::GravityField,         classes: Some(&[TrooperClass::ExoTech]) },

    GearRestrictions { id: GearID::EchoPulse,            classes: Some(&[TrooperClass::Decoy]) },
    GearRestrictions { id: GearID::HoloDoubler,          classes: Some(&[TrooperClass::Decoy]) },
    GearRestrictions { id: GearID::DoppelgangerSuit,     classes: Some(&[TrooperClass::Decoy]) },

    // General/All/No restriction
    GearRestrictions { id: GearID::NanoPatch,            classes: None },
    GearRestrictions { id: GearID::StimPack,             classes: None },
    GearRestrictions { id: GearID::FragGrenade,          classes: None },
    GearRestrictions { id: GearID::SmokeBomb,            classes: None },
    GearRestrictions { id: GearID::AdrenalineInjector,   classes: None },
    GearRestrictions { id: GearID::TrapKit,              classes: None },
];

// TODO: Go over all of GearStats and GEAR_STATS once effects have actually been implemented. The
// effects and flaws and everything shouldn't even be in this struct, they should be in the base
// Gear struct, although again there's no way to set it until the Armory's been set up.
// NOTE: Remember to change all "unlimited" to Usage::Unlimited or if false, then provide
// Usage::Limited(amount). Apparently, that design decision did not get to GPT, lol. At least the
// usage numbers were documented uner the usage attribute.

#[derive(Default, Debug, Copy, Clone)]
struct GearStats {
    id: GearID,
    uses: Option<u32>,
    unlimited: bool,
    action_cost: Option<char>, // 'A', 'F', or None
    effect: Option<Effect>,
    flaw: Option<EquipmentFlaw>,
}

static GEAR_STATS: &[GearStats] = &[
    // HEAVY
    GearStats {
        id:             GearID::ReinforcedPlating,
        uses:           None,
        unlimited:      true,
        action_cost:    None,
        effect:         Some(Effect::BuffAP { ap: 20, turns: 0, area: None }),
        flaw:           Some(EquipmentFlaw::CantCombineWith("Speed Boosters")),
    },
    GearStats {
        id:             GearID::AmmoFeederRig,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::ActionRefill),
        flaw:           Some(EquipmentFlaw::OnlyForBeltFed),
    },
    GearStats {
        id:             GearID::BlastShield,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("Place shield (blocks LOS, +40 AP)")),
        flaw:           Some(EquipmentFlaw::Custom("Stationary, one direction")),
    },
    GearStats {
        id:             GearID::ShockwaveGrenade,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("18 DMG, stuns bugs in 2-tile cone")),
        flaw:           Some(EquipmentFlaw::AttractsAggro),
    },

    // SCOUT
    GearStats {
        id:             GearID::CloakFieldUnit,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Cloak { turns: 1 }),
        flaw:           Some(EquipmentFlaw::Custom("Disables if attacking")),
    },
    GearStats {
        id:             GearID::GrappleLauncher,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::BuffMove { mv: 6, turns: 1 }),
        flaw:           Some(EquipmentFlaw::Cooldown { turns: 1 }),
    },
    GearStats {
        id:             GearID::EchoBeacon,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::AggroPull { turns: 2 }),
        flaw:           Some(EquipmentFlaw::DestructibleByEnemies { hp: 20 }),
    },
    GearStats {
        id:             GearID::HoloDecoy,
        uses:           None, // infinite, but with cooldown
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Decoy { duration: 2 }),
        flaw:           Some(EquipmentFlaw::OnlyOneActivePerSquad),
    },

    // MEDIC
    GearStats {
        id:             GearID::NanoMistInjector,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Heal { hp: 12, target: TargetType::Area }),
        flaw:           Some(EquipmentFlaw::Custom("No effect on armor")),
    },
    GearStats {
        id:             GearID::StabilizerDrone,
        uses:           Some(3),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Regen { hp_per_turn: 8, turns: 2 }),
        flaw:           Some(EquipmentFlaw::DestructibleByEnemies { hp: 20 }),
    },
    GearStats {
        id:             GearID::AntitoxinSpray,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::CleanseDebuffs { target: TargetType::Ally }),
        flaw:           None,
    },
    GearStats {
        id:             GearID::Painkillers,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::CleanseDebuffs { target: TargetType::Ally }),
        flaw:           Some(EquipmentFlaw::SelfDamage { dmg: 5, chance: 1.0 }),
    },

    // ENGINEER
    GearStats {
        id:             GearID::AutoTurret,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("Deploys turret (20 DMG/turn, 50 HP, 3-tile range)")),
        flaw:           Some(EquipmentFlaw::Cooldown { turns: 1 }),
    },
    GearStats {
        id:             GearID::PatchKit,
        uses:           Some(8),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::BuffAP { ap: 25, turns: 0, area: None }),
        flaw:           Some(EquipmentFlaw::NoEffectIfTargetAtMax),
    },
    GearStats {
        id:             GearID::SensorNode,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::RevealTraits),
        flaw:           Some(EquipmentFlaw::OneUsePerRun),
    },
    GearStats {
        id:             GearID::LaserTripwire,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("First bug to cross: 30 DMG + stun")),
        flaw:           Some(EquipmentFlaw::VisibleToEnemies),
    },
    GearStats {
        id:             GearID::PortableMinefield,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("Drop 3 mines, 15 DMG each")),
        flaw:           Some(EquipmentFlaw::FriendlyFirePossible),
    },
    GearStats {
        id:             GearID::ArcWelder,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Custom("Restore 10 AP to armor OR deal 12 energy DMG")),
        flaw:           Some(EquipmentFlaw::Custom("Short range, can't use twice on same")),
    },
    GearStats {
        id:             GearID::DetonationRemote,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Custom("Remotely trigger all mines/turrets (1-turn cooldown)")),
        flaw:           Some(EquipmentFlaw::Cooldown { turns: 1 }),
    },
    GearStats {
        id:             GearID::NanoGlueBomb,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::Immobilize { turns: 1 }),
        flaw:           Some(EquipmentFlaw::Custom("Can't stack on same bug")),
    },
    GearStats {
        id:             GearID::PlasmaCutter,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("Breach obstacles or 18 energy DMG to adjacent bug")),
        flaw:           Some(EquipmentFlaw::Cooldown { turns: 1 }),
    },

    // HANDLER
    GearStats {
        id:             GearID::HiveScanner,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::RevealTraits),
        flaw:           Some(EquipmentFlaw::Cooldown { turns: 3 }),
    },
    GearStats {
        id:             GearID::ChitinBait,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Pacify { turns: 1 }),
        flaw:           Some(EquipmentFlaw::NoEffectOnElite),
    },
    GearStats {
        id:             GearID::ConfusionCollar,
        uses:           Some(3),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Confuse { turns: 1 }),
        flaw:           Some(EquipmentFlaw::NoEffectOnElite),
    },
    GearStats {
        id:             GearID::BugPheromoneBomb,
        uses:           Some(5),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::AggroPull { turns: 1 }),
        flaw:           Some(EquipmentFlaw::FriendlyFirePossible),
    },

    // EXO-TECH
    GearStats {
        id:             GearID::ShellPack,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::RecoverAmmoOnCrit),
        flaw:           Some(EquipmentFlaw::Custom("Can't use twice per weapon/mission")),
    },
    GearStats {
        id:             GearID::PlasmaShield,
        uses:           Some(3),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::BuffAP { ap: 25, turns: 2, area: None }),
        flaw:           Some(EquipmentFlaw::Custom("Fades if Exo-Tech moves")),
    },
    GearStats {
        id:             GearID::UltraShredRounds,
        uses:           Some(2),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::IgnoreArmor),
        flaw:           Some(EquipmentFlaw::CantCombineWith("Shell Pack")),
    },
    GearStats {
        id:             GearID::GravityField,
        uses:           Some(2),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::BuffMove { mv: -50, turns: 2 }), // Negative to show half-move? (Up to you)
        flaw:           Some(EquipmentFlaw::OneUsePerRun),
    },

    // DECOY
    GearStats {
        id:             GearID::EchoPulse,
        uses:           Some(2),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Custom("All bugs lose 1 action next turn")),
        flaw:           Some(EquipmentFlaw::OneUsePerRun),
    },
    GearStats {
        id:             GearID::HoloDoubler,
        uses:           Some(3),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::HoloDouble { turns: 1 }),
        flaw:           Some(EquipmentFlaw::MovementPenalty { penalty: -2, turns: 2 }),
    },
    GearStats {
        id:             GearID::DoppelgangerSuit,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Custom("Copy another trooper’s class for 1 turn")),
        flaw:           Some(EquipmentFlaw::AttractsAggro),
    },

    // GENERAL/ALL
    GearStats {
        id:             GearID::NanoPatch,
        uses:           None,
        unlimited:      true,
        action_cost:    Some('F'),
        effect:         Some(Effect::Regen { hp_per_turn: 2, turns: 99 }),
        flaw:           Some(EquipmentFlaw::OnlyOneActivePerSquad),
    },
    GearStats {
        id:             GearID::StimPack,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::BuffMove { mv: 2, turns: 1 }),
        flaw:           Some(EquipmentFlaw::SelfDamage { dmg: 10, chance: 1.0 }),
    },
    GearStats {
        id:             GearID::FragGrenade,
        uses:           Some(6),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::AoE { dmg: 25, aoe: Area::Immediate }),
        flaw:           Some(EquipmentFlaw::AttractsAggro),
    },
    GearStats {
        id:             GearID::SmokeBomb,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::Cloak { turns: 1 }),
        flaw:           Some(EquipmentFlaw::PoorAccuracy { penalty: -20 }),
    },
    GearStats {
        id:             GearID::AdrenalineInjector,
        uses:           Some(1),
        unlimited:      false,
        action_cost:    Some('F'),
        effect:         Some(Effect::Revive { hp: 25 }),
        flaw:           Some(EquipmentFlaw::StunnedAfterUse { turns: 1 }),
    },
    GearStats {
        id:             GearID::TrapKit,
        uses:           Some(4),
        unlimited:      false,
        action_cost:    Some('A'),
        effect:         Some(Effect::Trap { duration: 2 }),
        flaw:           Some(EquipmentFlaw::OnlyOneActivePerSquad),
    },
];

fn get_gear_stats(id: GearID) -> GearStats {
    GEAR_STATS.iter().find(|g| g.id == id).expect(&format!("Invalid Gear ID: {:?}", id)).clone()
}

#[derive(Default, Clone, Debug)]
pub struct Gear {
    id: GearID,
    info: GearInfo,
    stats: GearStats,
    effect: Option<Effect>,
    flaw: Option<EquipmentFlaw>,
}

impl Gear {
    fn new(id: GearID) -> Self {
        let info = get_gear_info(id);
        let stats = get_gear_stats(id);

        Gear {
            id,
            info,
            stats,
            effect: None,
            flaw: None,
        }
    }
}

pub struct Armory;

impl Armory {
    fn allowed_for_class(id: EquipmentID, class: TrooperClass) -> bool {
        match id {
            EquipmentID::WeaponID(wid) => {
                WEAPON_RESTRICTIONS
                    .iter()
                    .find(|r| r.id == wid)
                    .map(|r| match r.classes {
                        Some(classes) => classes.contains(&class),
                        None => true,
                    })
                    .unwrap_or(true)
            },
            EquipmentID::GearID(gid) => {
                GEAR_RESTRICTIONS
                    .iter()
                    .find(|g| g.id == gid)
                    .map(|g| match g.classes {
                        Some(classes) => classes.contains(&class),
                        None => true,
                    })
                    .unwrap_or(true)
            }
        }
    }

    fn fetch_allowed_weapons(class: TrooperClass) -> Vec<WeaponID> {
        ALL_WEAPON_IDS
            .iter()
            .copied()
            .filter(|wid| Self::allowed_for_class(EquipmentID::WeaponID(*wid), class))
            .collect()
    }

    fn load_weapons(class: TrooperClass) -> Vec<Weapon> {
        let allowed_weapons = Self::fetch_allowed_weapons(class);
        let mut weapons = vec![];

        for (_, w) in allowed_weapons.into_iter().enumerate() {
            let weapon = Weapon::new(w);
            weapons.push(weapon);
        }

        weapons
    }

    pub fn print_class_weapons(class: TrooperClass) {
        let weapons: Vec<Weapon> = Self::load_weapons(class);
        println!("");
        println!("Trooper Class: {:?}", class);
        println!("");
        for (_, weapon) in weapons.into_iter().enumerate() {
            println!("<<<<<<<<< {:?} >>>>>>>>>", weapon.id);
            println!("ID: {:?}", weapon.id);
            println!("Name: {:?}", weapon.info.name);
            println!("Type: {:?}", weapon.info.r#type);
            println!("Description (Flavor): {:?} ({})", weapon.info.description, weapon.info.flavor);
            println!("Info: {:?}", weapon.info);
            println!("Stats: {:?}", weapon.stats);
            println!("Effect: {:?}", weapon.effect);
            println!("Flaw: {:?}", weapon.flaw);
            println!();
        }
    }

    fn fetch_allowed_gear(class: TrooperClass) -> Vec<GearID> {
        ALL_GEAR_IDS
            .iter()
            .copied()
            .filter(|gid| Self::allowed_for_class(EquipmentID::GearID(*gid), class))
            .collect()
    }

    fn load_gear(class: TrooperClass) -> Vec<Gear> {
        let allowed_gear = Self::fetch_allowed_gear(class);
        let mut gear = vec![];

        for (_, g) in allowed_gear.into_iter().enumerate() {
            let item = Gear::new(g);
            gear.push(item);
        }

        gear
    }

    pub fn print_class_gear(class: TrooperClass) {
        let gear: Vec<Gear> = Self::load_gear(class);
        println!("");
        println!("Trooper Class: {:?}", class);
        println!("");
        for (_, item) in gear.into_iter().enumerate() {
            println!("+++++++++++ {:?} +++++++++++", item.id);
            println!("Name: {:?}", item.info.name);
            println!("Type: {:?}", item.info.r#type);
            println!("Description (Flavor): {:?} ({})", item.info.description, item.info.flavor);
            println!("Stats: {:?}", item.stats);
            println!("Effect: {:?}", item.effect);
            println!("Flaw: {:?}", item.flaw);
            println!();
        }
    }
}

pub struct Loadout {
    weapons: Vec<Weapon>,
    gear: Vec<Gear>,
}

impl Loadout {
    pub fn new(class: TrooperClass) -> Self {
        let weapons = Armory::load_weapons(class);
        let gear = Armory::load_gear(class);

        Loadout {
            weapons,
            gear,
        }
    }
}
