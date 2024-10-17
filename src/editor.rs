use nih_plug::prelude::Editor;
use nih_plug_vizia::{assets, create_vizia_editor, vizia::prelude::*, widgets::{ParamSlider, ResizeHandle}, ViziaState, ViziaTheming};
use std::sync::Arc;


use crate::{param_switch_button::{ParamSwitchButtonExt, ParamSwitchButton}, EqualiserParams, FilterTypes};

#[derive(Lens)]
struct Data {
    params: Arc<EqualiserParams>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 400))
}

pub(crate) fn create(
    params: Arc<EqualiserParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            Label::new(cx, "Single Band EQ")
                .font_family(vec![FamilyOwned::Name(String::from(assets::NOTO_SANS))])
                .font_weight(FontWeightKeyword::Thin)
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(0.0));
            Label::new(cx, match params.band.filter_type.value() {
                FilterTypes::HPF => "High Pass Filter",
                FilterTypes::LPF => "Low Pass Filter",
                FilterTypes::PEAK => "Peak Filter",
                FilterTypes::HS => "High Shelf",
                FilterTypes::LS => "Low Shelf",
            });
            
            Label::new(cx, "Frequency");
            ParamSlider::new(cx, Data::params, |params| &params.band.frequency);

            
            Label::new(cx, "Q");
            ParamSlider::new(cx, Data::params, |params| &params.band.q);

            VStack::new(cx, |cx| {
                Label::new(cx, "Gain");
                ParamSlider::new(cx, Data::params, |params| &params.band.gain);
            }).visibility(Data::params.map(|p| {
                match p.band.filter_type.value() {
                    FilterTypes::HPF | FilterTypes::LPF => false,
                    FilterTypes::PEAK | FilterTypes::LS | FilterTypes::HS => true,
                }
            }));


                ParamSwitchButton::new(
                    cx,
                    Data::params,
                    |params| &params.band.filter_type,
                    FilterTypes::PEAK
                )
                .disabled(Data::params.map(|p| p.band.filter_type.value() == FilterTypes::PEAK))
                .with_label("Peak");
                ParamSwitchButton::new(
                    cx,
                    Data::params,
                    |params| &params.band.filter_type, 
                    FilterTypes::HPF
                )
                .disabled(Data::params.map(|p| p.band.filter_type.value() == FilterTypes::HPF))
                .with_label("High Pass");
                ParamSwitchButton::new(
                    cx,
                    Data::params,
                    |params| &params.band.filter_type,
                    FilterTypes::LPF
                )
                .disabled(Data::params.map(|p| p.band.filter_type.value() == FilterTypes::LPF))
                .with_label("Low Pass");
                ParamSwitchButton::new(
                    cx,
                    Data::params,
                    |params| &params.band.filter_type,
                    FilterTypes::HS
                )
                .disabled(Data::params.map(|p| p.band.filter_type.value() == FilterTypes::HS))
                .with_label("High Shelf");
                ParamSwitchButton::new(
                    cx,
                    Data::params,
                    |params| &params.band.filter_type,
                    FilterTypes::LS
                )
                .disabled(Data::params.map(|p| p.band.filter_type.value() == FilterTypes::LS))
                .with_label("Low Shelf");
                
        })
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));

        ResizeHandle::new(cx);
    })
}
