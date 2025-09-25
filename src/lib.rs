pub(crate) mod reader;
pub mod soundfont;
#[allow(missing_docs)]
#[allow(clippy::module_inception)]
pub mod synthesizer;

pub(crate) mod utils;

pub mod prelude {
    pub use crate::{
        soundfont::{instrument::*, preset::*, *},
        synthesizer::*,
    };

    pub(crate) use crate::{reader::*, soundfont::generator::*};
    pub use std::io::Read;
}
