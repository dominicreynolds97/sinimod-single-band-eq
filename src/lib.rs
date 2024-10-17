use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;
use core::f32;
use std::{f32::consts::PI, sync::Arc};
use nih_plug_vizia::vizia::prelude::*;

mod editor;
mod param_switch_button;

pub struct Equaliser {
    params: Arc<EqualiserParams>,
}

#[derive(Params, Lens)]
pub struct EqualiserParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[nested(group = "Band")]
    pub band: BandParams,
}

#[derive(Enum, PartialEq)]
pub enum FilterTypes {
    #[id="peak"]
    #[name="Peak"]
    PEAK,
    #[id="hpf"]
    #[name="High Pass"]
    HPF,
    #[id="lpf"]
    #[name="Low Pass"]
    LPF,
    #[id="ls"]
    #[name="Low Shelf"]
    LS,
    #[id="hs"]
    #[name="High Shelf"]
    HS,
}

#[derive(Params, Lens)]
pub struct BandParams {
    #[id = "frequency"] // Center frequency
    pub frequency: FloatParam,

    #[id = "gain"] // dB - Boost/Cut gain
    pub gain: FloatParam,

    #[id = "q"] // Quality factor
    pub q: FloatParam,

    #[id = "filter-type"]
    pub filter_type: EnumParam<FilterTypes>,
}

struct FilterCoeffs {
    a: [f32; 3],
    b: [f32; 3],
}

impl BandParams {
    fn new(f: f32) -> Self {
        BandParams {
            frequency: FloatParam::new(
                "frequency",
                f,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-1.0)
                }
            ),
            gain: FloatParam::new(
                "gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            q: FloatParam::new(
                "q",
                3.0,
                FloatRange::Linear { min: 0.01, max: 20.0 }
            ),
            filter_type: EnumParam::new(
                "filter-type",
                FilterTypes::PEAK
            ),
        }
    }

    fn get_a(&self) -> f32 {
        10.0_f32.powf(&self.gain.value() / 20.0).sqrt()
    }

    fn get_w(&self, sample_rate: f32) -> f32 {
        2.0 * PI * (&self.frequency.value() / sample_rate)
    }

    fn get_alpha(&self, w: f32) -> f32 {
        w.sin() / (2.0 * &self.q.value())
    }

    fn filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        match self.filter_type.value() {
            FilterTypes::PEAK => self.peak_filter_params(sample_rate),
            FilterTypes::HPF => self.hpf_filter_params(sample_rate),
            FilterTypes::LPF => self.lpf_filter_params(sample_rate),
            FilterTypes::HS => self.hs_filter_params(sample_rate),
            FilterTypes::LS => self.ls_filter_params(sample_rate),
        }
    }

    fn ls_filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        let w = self.get_w(sample_rate);
        let a = self.get_a();
        let alpha = self.get_alpha(w);
        FilterCoeffs {
            a: [
                (a + 1.0) + ((a - 1.0) * w.cos()) + (2.0 * (a * alpha).sqrt()),
                -2.0 * ((a - 1.0) + ((a + 1.0) * w.cos())),
                (a + 1.0) + ((a - 1.0) * w.cos()) - (2.0 * (a * alpha).sqrt()),
            ],
            b: [
                a * ((a + 1.0) - ((a - 1.0) * w.cos()) + (2.0 * (a * alpha).sqrt())),
                2.0 * a * ((a - 1.0) - ((a + 1.0) * w.cos())),
                a * ((a + 1.0) - ((a - 1.0) * w.cos()) - (2.0 * (a * alpha).sqrt())),
            ],
        }
    }
    fn hs_filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        let w = self.get_w(sample_rate);
        let a = self.get_a();
        let alpha = self.get_alpha(w);
        FilterCoeffs {
            a: [
                (a + 1.0) - ((a - 1.0) * w.cos()) + (2.0 * (a * alpha).sqrt()),
                2.0 * ((a - 1.0) - ((a + 1.0) * w.cos())),
                (a + 1.0) - ((a - 1.0) * w.cos()) - (2.0 * (a * alpha).sqrt()),
            ],
            b: [
                a * ((a + 1.0) + ((a - 1.0) * w.cos()) + (2.0 * (a * alpha).sqrt())),
                -2.0 * a * ((a - 1.0) + ((a + 1.0) * w.cos())),
                a * ((a + 1.0) + ((a - 1.0) * w.cos()) - (2.0 * (a * alpha).sqrt())),
            ],
        }
    }

    fn hpf_filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        let w = self.get_w(sample_rate);
        let alpha = self.get_alpha(w);
        FilterCoeffs {
            a: [
                1.0 + alpha,
                -2.0 * w.cos(),
                1.0 - alpha,
            ],
            b: [
                (1.0 + w.cos()) / 2.0,
                -1.0 * (1.0 + w.cos()),
                (1.0 + w.cos()) / 2.0,
            ],
        }
    }

    fn lpf_filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        let w = self.get_w(sample_rate);
        let alpha = self.get_alpha(w);
        FilterCoeffs {
            a: [
                1.0 + alpha,
                -2.0 * w.cos(),
                1.0 - alpha,
            ],
            b: [
                (1.0 - w.cos()) / 2.0,
                1.0 - w.cos(),
                (1.0 - w.cos()) / 2.0,
            ],
        }
    }

    fn peak_filter_params(&self, sample_rate: f32) -> FilterCoeffs {
        let w = self.get_w(sample_rate);
        let alpha = self.get_alpha(w);
        let a = self.get_a();
        FilterCoeffs {
            a: [
                1.0 + (alpha / a),
                -2.0 * w.cos(),
                1.0 - (alpha / a)
            ],
            b: [
                1.0 + (alpha * a),
                -2.0 * w.cos(),
                1.0 - (alpha * a)
            ],
        }
    }
}

impl Default for Equaliser {
    fn default() -> Self {
        Self {
            params: Arc::new(EqualiserParams::default()),
        }
    }
}

impl Default for EqualiserParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            band: BandParams::new(400.0),
        }
    }
}

impl Plugin for Equaliser {
    const NAME: &'static str = "Single Band EQ";
    const VENDOR: &'static str = "Sinimod Plugins";
    const URL: &'static str = "https://sinimod.com/plugins";
    const EMAIL: &'static str = "dominic@sinimod.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames::const_default(),
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    
    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
//        create_egui_editor(
//            self.params.editor_state.clone(),
//            (),
//            |_, _| {},
//            move |egui_ctx, setter, _state| {
//                egui::CentralPanel::default().show(egui_ctx, |ui| {
//                    ui.label("Frequency");
//                    ui.add(ParamSlider::for_param(&params.band.frequency, setter));
//
//                    ui.label("Gain");
//                    ui.add(ParamSlider::for_param(&params.band.gain, setter));
//
//                    ui.label("Q");
//                    ui.add(ParamSlider::for_param(&params.band.q, setter));
//                });
//            },
//        )
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>
    ) -> bool {
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let coeffs = &self.params.band.filter_params(41000.0);
        let channels = buffer.channels();
        let mut x_prev: Vec<[f32; 3]> = Vec::new();
        let mut y_prev: Vec<[f32; 3]> = Vec::new();

        for _i in 0..channels {
            x_prev.push([0.0,0.0,0.0]);
            y_prev.push([0.0,0.0,0.0]);
        };

        for channel_samples in buffer.iter_samples() {
            let mut c = 0;
            for channel_sample in channel_samples {
                let x = x_prev.get_mut(c).expect("channels should always be in bounds");
                let y = y_prev.get_mut(c).expect("channels should always be in bounds");

                x[2] = x[1];
                x[1] = x[0];
                x[0] = channel_sample.clone();
                y[2] = y[1];
                y[1] = y[0];
                
                *channel_sample = (coeffs.b[0] / coeffs.a[0]) * x[0];
                *channel_sample += (coeffs.b[1] / coeffs.a[0]) * x[1];
                *channel_sample += (coeffs.b[2] / coeffs.a[0]) * x[2];
                *channel_sample -= (coeffs.a[1] / coeffs.a[0]) * y[1];
                *channel_sample -= (coeffs.a[2] / coeffs.a[0]) * y[2];

                c += 1;
                y[0] = channel_sample.clone();
            }
        }
        
        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl Vst3Plugin for Equaliser {
    const VST3_CLASS_ID: [u8; 16] = *b"SinimodEQPlugino";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx, Vst3SubCategory::Tools];
}

nih_export_vst3!(Equaliser);

