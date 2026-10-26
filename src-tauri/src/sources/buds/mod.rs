use crate::core::source::{ActivitySource, SourceContext};

#[derive(Default)]
pub struct BudsSource;

impl ActivitySource for BudsSource {
    fn name(&self) -> &'static str {
        "buds"
    }

    fn start(self: Box<Self>, ctx: SourceContext) {
        ctx.idle_until_shutdown();
    }
}
