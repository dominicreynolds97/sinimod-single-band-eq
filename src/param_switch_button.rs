//! A toggleable button that integrates with NIH-plug's [`Param`] types.

use nih_plug::prelude::Param;
use nih_plug_vizia::{vizia::{binding::Lens, prelude::*, context::{Context, EventContext}, events::Event, input::MouseButton, view::{Handle, View}, views::{Binding, Label}, window::WindowEvent}, widgets::param_base::ParamWidgetBase};

use crate::FilterTypes;


/// A toggleable button that integrates with NIH-plug's [`Param`] types. Only makes sense with
/// [`BoolParam`][nih_plug::prelude::BoolParam]s. Clicking on the button will toggle between the
/// parameter's minimum and maximum value. The `:checked` pseudoclass indicates whether or not the
/// button is currently pressed.
#[derive(Lens)]
pub struct ParamSwitchButton {
    param_base: ParamWidgetBase,

    // These fields are set through modifiers:
    /// Whether or not to listen to scroll events for changing the parameter's value in steps.
    use_scroll_wheel: bool,
    /// A specific label to use instead of displaying the parameter's value.
    label_override: Option<String>,
    value: FilterTypes,

    /// The number of (fractional) scrolled lines that have not yet been turned into parameter
    /// change events. This is needed to support trackpads with smooth scrolling.
    scrolled_lines: f32,
}

impl ParamSwitchButton {
    /// Creates a new [`ParamButton`] for the given parameter. See
    /// [`ParamSlider`][super::ParamSlider] for more information on this function's arguments.
    pub fn new<L, Params, P, FMap>(
        cx: &mut Context,
        params: L,
        params_to_param: FMap,
        value: FilterTypes,
    ) -> Handle<Self>
    where
        L: Lens<Target = Params> + Clone,
        Params: 'static,
        P: Param + 'static,
        FMap: Fn(&Params) -> &P + Copy + 'static,
    {
        Self {
            param_base: ParamWidgetBase::new(cx, params.clone(), params_to_param),

            use_scroll_wheel: true,
            label_override: None,
            value,

            scrolled_lines: 0.0,
        }
        .build(
            cx,
            ParamWidgetBase::build_view(params.clone(), params_to_param, move |cx, param_data| {
                Binding::new(cx, Self::label_override, move |cx, label_override| {
                    match label_override.get(cx) {
                        Some(label_override) => Label::new(cx, &label_override),
                        None => Label::new(cx, param_data.param().name()),
                    }
                    .hoverable(false);
                })
            }),
        )
        // We'll add the `:checked` pseudoclass when the button is pressed
        // NOTE: We use the normalized value _with modulation_ for this. There's no convenient way
        //       to show both modulated and unmodulated values here.
        .checked(ParamWidgetBase::make_lens(
            params,
            params_to_param,
            |param| param.modulated_normalized_value() >= 0.5,
        ))
    }

    /// Set the parameter's normalized value to either 0.0 or 1.0 depending on its current value.
    fn toggle_value(&self, cx: &mut EventContext) {
        //let current_value = self.param_base.unmodulated_normalized_value();
        let new_value = match self.value {
            FilterTypes::PEAK => 0.0,
            FilterTypes::HPF => 0.25,
            FilterTypes::LPF => 0.5,
            FilterTypes::LS => 0.75,
            FilterTypes::HS => 1.0,
        };

        self.param_base.begin_set_parameter(cx);
        self.param_base.set_normalized_value(cx, new_value);
        self.param_base.end_set_parameter(cx);
    }
}

impl View for ParamSwitchButton {
    fn element(&self) -> Option<&'static str> {
        Some("param-button")
    }

    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event, meta| match window_event {
            // We don't need special double and triple click handling
            WindowEvent::MouseDown(MouseButton::Left)
            | WindowEvent::MouseDoubleClick(MouseButton::Left)
            | WindowEvent::MouseTripleClick(MouseButton::Left) => {
                self.toggle_value(cx);
                meta.consume();
            }
            WindowEvent::MouseScroll(_scroll_x, scroll_y) if self.use_scroll_wheel => {
                // With a regular scroll wheel `scroll_y` will only ever be -1 or 1, but with smooth
                // scrolling trackpads being a thing `scroll_y` could be anything.
                self.scrolled_lines += scroll_y;

                if self.scrolled_lines.abs() >= 1.0 {
                    self.param_base.begin_set_parameter(cx);

                    if self.scrolled_lines >= 1.0 {
                        self.param_base.set_normalized_value(cx, 1.0);
                        self.scrolled_lines -= 1.0;
                    } else {
                        self.param_base.set_normalized_value(cx, 0.0);
                        self.scrolled_lines += 1.0;
                    }

                    self.param_base.end_set_parameter(cx);
                }

                meta.consume();
            }
            _ => {}
        });
    }
}

pub trait ParamSwitchButtonExt {
    fn with_label(self, value: impl Into<String>) -> Self;
}

impl ParamSwitchButtonExt for Handle<'_, ParamSwitchButton> {
    fn with_label(self, value: impl Into<String>) -> Self {
        self.modify(|param_button: &mut ParamSwitchButton| {
            param_button.label_override = Some(value.into())
        })
    }
}
