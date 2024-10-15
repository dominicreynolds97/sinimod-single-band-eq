use nih_plug::prelude::*;

use sinimod_single_band_eq::Equaliser; 

fn main() {
    nih_export_standalone::<Equaliser>();
}
