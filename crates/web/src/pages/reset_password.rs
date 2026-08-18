use topcoat::{Result, router::page, view::view};

use crate::components::{
    auth_panel::auth_panel,
    form_field::{FieldSpec, form_field},
    form_status::form_status,
    submit_button::submit_button,
};

#[page("/reset-password")]
pub(crate) async fn reset_password_page() -> Result {
    view! {
        auth_panel(
            title: "Choose a new password",
            description: "Use a password you haven’t used for this account before.",
            <form id="reset-password-form" class="flex flex-col gap-5" novalidate="novalidate">
                <label class="sr-only" for="reset-username">"Account email"</label>
                <input
                    id="reset-username"
                    class="sr-only"
                    name="username"
                    type="email"
                    autocomplete="username"
                    tabindex="-1"
                >
                form_field(
                    field: FieldSpec::new(
                        "reset-password",
                        "New password",
                        "password",
                        "new-password",
                        "Your new password",
                    ).hint("8–128 characters").min_length(8),
                )
                form_field(
                    field: FieldSpec::new(
                        "confirm-reset-password",
                        "Confirm new password",
                        "password",
                        "new-password",
                        "Repeat your new password",
                    ).min_length(8),
                )
                form_status(id: "reset-password-status")
                submit_button(label: "Reset password", pending_label: "Resetting password…")
            </form>
            <p class="mt-6 text-center text-sm text-muted-foreground">
                <a class="font-medium text-foreground underline underline-offset-4" href="/forgot-password">
                    "Request another reset link"
                </a>
            </p>
        )
    }
}
