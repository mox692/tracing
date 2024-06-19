use std::mem::size_of;

use tracing::trace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    // initialize buffers, etc
    let layer = tracing_flight_recorder::Layer::new();

    println!("{}", size_of::<[u8; 90000]>());
    tracing_subscriber::registry().with(layer).init();

    let jh = std::thread::spawn(|| {
        // logging
        loop {
            trace!("s");
        }
    });
    // loop {
    //     trace!("p");
    // }

    jh.join().unwrap();
}
