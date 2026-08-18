use topcoat::{
    Result,
    view::{attributes, component, view},
};

use super::alert::{AlertVariant, alert};

#[component]
pub(crate) async fn form_status(id: &'static str, #[default] status: bool) -> Result {
    let role = if status { "status" } else { "alert" };

    view! {
        alert(
            variant: AlertVariant::Destructive,
            attrs: attributes! {
                id=(id)
                class="outline-none data-[tone=success]:!border-emerald-200 data-[tone=success]:!text-emerald-800"
                role=(role)
                aria-live="polite"
                tabindex="-1"
                hidden="hidden"
            },
            <span class="col-start-2" data-status-message="true"></span>
        )
    }
}
