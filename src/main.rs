//TODO: Create a tests/ folder and move all debug printing/info and whatnot there, after creating
//the turn_handler and effect matchup. Cause then the output will be actually for debugging instead
//of the only thing that outputs.
//NOTE: Perhaps make a tests/ folder and a debug/ folder? That way there could actually be tests in
//tests/ and all the debug output could be handled in the debug/ folder

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
