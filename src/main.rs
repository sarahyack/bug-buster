// TODO: Flesh out LoreMaster on the Lore branch

mod utils;
mod debug;
mod bugs;
mod hive;
mod battlefield;
mod troopers;
mod armory;
mod tui;

use debug::LOG;
use bugs::Broodmother;
use troopers::Commander;
use hive::Cartographer;
// use armory::Armory;

fn main() {
    println!("Hello, world!");
    let wave = Broodmother::spawn_test_wave(5);
    Broodmother::debug_wave(&wave);
    let team = Commander::test_trooper_creation(3);
    Commander::spawn_troopers(&team);
    Commander::log_team_gear(team);
    Cartographer::spawn_chambers(15);
    LOG.lock().unwrap().print_all();
}
