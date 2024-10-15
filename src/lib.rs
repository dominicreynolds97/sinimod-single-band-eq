use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, widgets::ParamSlider, EguiState};
use std::sync::Arc;

pub struct Equaliser {
    params: Arc<EqualiserParams>,
}

#[derive(Params)]
pub struct EqualiserParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "frequency"]
    pub frequency: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "q"]
    pub q: FloatParam,
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
            editor_state: EguiState::from_size(300, 180),
            frequency: FloatParam::new(
                "frequency",
                400.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-10.0)
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
                1.0,
                FloatRange::Skewed { min: 0.01, max: 20.0, factor: 0.4 }
            )
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
        let params = self.params.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("Frequency");
                    ui.add(ParamSlider::for_param(&params.frequency, setter));

                    ui.label("Gain");
                    ui.add(ParamSlider::for_param(&params.gain, setter));

                    ui.label("Q");
                    ui.add(ParamSlider::for_param(&params.q, setter));
                });
            },
        )
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
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
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

