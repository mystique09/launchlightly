use topcoat::{
    Result,
    view::{attributes, component, view},
};

use super::button::{ButtonSize, button};

#[component]
pub(crate) async fn submit_button(label: &'static str, pending_label: &'static str) -> Result {
    view! {
        button(
            size: ButtonSize::Lg,
            attrs: attributes! {
                class="w-full"
                type="submit"
                data-default-label=(label)
                data-pending-label=(pending_label)
            },
            <span data-button-label="true">(label)</span>
        )
    }
}
