mod state;
use state::State;

fn main() {
    /* ----- ----- */
    pollster::block_on(State::run());
}
