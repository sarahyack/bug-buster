mod bugs;
mod environments;
mod troopers;
mod tui;

use bugs::Broodmother;

fn main() {
    println!("Hello, world!");
    let wave = Broodmother::spawn_test_wave(5);
    Broodmother::debug_wave(&wave);
}
