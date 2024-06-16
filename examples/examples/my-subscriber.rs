//! A [`tracing-subscriber`] Layer which outputs to `stdout`.
//!
//! See the documentation on [`Layer`] for more details.
//!
//! [`tracing-subscriber`]: tracing_subscriber
use tracing::{event, span, Collect, Level};
use tracing_core::Interest;
use tracing_subscriber::{registry::LookupSpan, Subscribe};

#[must_use = "A Layer does nothing if it is not added to a registry."]
pub fn mylayer() -> MyLayer {
    MyLayer::new()
}

pub struct MyLayer {}

impl MyLayer {
    #[must_use = "A Layer does nothing if it is not added to a registry."]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MyLayer {
    #[must_use = "A Layer does nothing if it is not added to a registry."]
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Collect> Subscribe<C> for MyLayer {
    fn on_event(
        &self,
        _event: &event::Event<'_>,
        _ctx: tracing_subscriber::subscribe::Context<'_, C>,
    ) {
        println!("event called!")
    }
}

fn main() {
    use tracing_subscriber::prelude::*;

    let layer = mylayer();

    tracing_subscriber::registry().with(layer).init();

    event!(Level::INFO, "event");
}

// if enabled {
//     (|value_set: ::tracing::field::ValueSet| {
//         let meta = __CALLSITE.metadata();
//         ::tracing::Event::dispatch(meta, &value_set);
//     })({
//         #[allow(unused_imports)]
//         use ::tracing::field::{debug, display, Value};
//         let mut iter = __CALLSITE.metadata().fields().iter();
//         __CALLSITE
//             .metadata()
//             .fields()
//             .value_set(
//                 &[
//                     (
//                         &::tracing::__macro_support::Iterator::next(&mut iter)
//                             .expect("FieldSet corrupted (this is a bug)"),
//                         ::tracing::__macro_support::Option::Some(
//                             &format_args!("event") as &dyn Value,
//                         ),
//                     ),
//                 ],
//             )
//     });
// } else {
// }
