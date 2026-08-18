use topcoat::{
    Result,
    icon::{icon, iconify::iconify_icon},
    view::{attributes, component, view},
};

use super::{
    button::{ButtonSize, ButtonVariant, button},
    input::input,
    label::label,
};

pub(crate) struct FieldSpec {
    id: &'static str,
    label_text: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
    hint: &'static str,
    min_length: Option<usize>,
}

impl FieldSpec {
    pub(crate) const fn new(
        id: &'static str,
        label_text: &'static str,
        input_type: &'static str,
        autocomplete: &'static str,
        placeholder: &'static str,
    ) -> Self {
        Self {
            id,
            label_text,
            input_type,
            autocomplete,
            placeholder,
            hint: "",
            min_length: None,
        }
    }

    pub(crate) const fn hint(mut self, hint: &'static str) -> Self {
        self.hint = hint;
        self
    }

    pub(crate) const fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }
}

#[component]
pub(crate) async fn form_field(field: FieldSpec) -> Result {
    let hint_id = format!("{}-hint", field.id);

    view! {
        <div class="flex flex-col gap-2">
            <div class="flex items-baseline justify-between gap-3">
                label(attrs: attributes! { for=(field.id) }, (field.label_text))
                if !field.hint.is_empty() {
                    <span id=(&hint_id) class="text-xs text-muted-foreground">(field.hint)</span>
                }
            </div>
            <div class="relative">
                input(
                    attrs: attributes! {
                        id=(field.id)
                        name=(field.id)
                        type=(field.input_type)
                        autocomplete=(field.autocomplete)
                        placeholder=(field.placeholder)
                        required="required"
                        class=(if field.input_type == "password" { "h-11 pr-12" } else { "h-11" })
                        if !field.hint.is_empty() {
                            aria-describedby=(&hint_id)
                        }
                        if let Some(min_length) = field.min_length {
                            minlength=(min_length)
                        }
                        if field.input_type == "password" {
                            maxlength="128"
                        }
                    },
                )
                if field.input_type == "password" {
                    button(
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        attrs: attributes! {
                            type="button"
                            class="absolute top-1 right-1"
                            aria-label="Show password"
                            aria-controls=(field.id)
                            aria-pressed="false"
                            data-password-toggle=(field.id)
                        },
                        <span data-password-visible-icon="true">
                            icon(data: iconify_icon!("feather:eye"))
                        </span>
                        <span data-password-hidden-icon="true" hidden="hidden">
                            icon(data: iconify_icon!("feather:eye-off"))
                        </span>
                    )
                }
            </div>
        </div>
    }
}
