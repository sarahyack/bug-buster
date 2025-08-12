// TODO: Flesh out LoreMaster on the Lore branch

mod utils;
mod debug;
mod bugs;
mod hive;
mod battlefield;
mod combat;
mod troopers;
mod armory;
mod tui;

use battlefield::Overwatch;

fn main() {
    println!("Hello, world!");
    let mut ovw = Overwatch::new();
    ovw.start_game();
}
