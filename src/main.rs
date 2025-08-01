mod bugs;
mod environments;
mod battlefield;
mod troopers;
mod armory;
mod tui;
mod utils;

use bugs::Broodmother;
use troopers::Commander;

fn main() {
    println!("Hello, world!");
    let wave = Broodmother::spawn_test_wave(5);
    Broodmother::debug_wave(&wave);
    let team = Commander::test_trooper_creation(3);
    Commander::spawn_troopers(&team);
}
