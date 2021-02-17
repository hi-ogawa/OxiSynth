#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate derivative;

#[macro_use]
mod macros {
    macro_rules! gerr {
        ($err:expr, $fmt_string:expr, $( $arg:expr ),*) => {
            { println!($fmt_string, $( $arg ),*); 0 }
        }
    }
}

pub mod channel;
pub mod chorus;
pub mod conv;
pub mod dsp_float;
pub mod gen;
pub mod modulator;
pub mod reverb;
pub mod settings;
pub mod sfloader;
pub mod soundfont;
pub mod synth;
pub mod tuning;
pub mod voice;

pub mod fileapi;
