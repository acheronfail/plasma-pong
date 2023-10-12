mod cli;
mod engine;
mod fps;
mod renderer;
mod state;
mod window;

use clap::Parser;
use cli::Cli;
use engine::Engine;
use state::State;

pub fn main() -> ! {
    Engine::run(Cli::parse());
}
