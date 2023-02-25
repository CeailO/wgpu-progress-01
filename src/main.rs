mod state;
use state::State;

mod vertex_lib;

fn main() {
    /* ----- ----- */
    pollster::block_on(State::run());
}
