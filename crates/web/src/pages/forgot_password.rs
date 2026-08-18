use topcoat::{Result, router::page, view::view};

use crate::components::{
    auth_panel::auth_panel,
    form_field::{FieldSpec, form_field},
    form_status::form_status,
    submit_button::submit_button,
};

#[page("/forgot-password")]
pub(crate) async fn forgot_password_page() -> Result {
    view! {
        auth_panel(
            title: "Reset your password",
            description: "Enter your account email and we’ll create a secure reset link.",
            <form id="forgot-password-form" class="flex flex-col gap-5" novalidate="novalidate">
                form_field(
                    field: FieldSpec::new(
                        "email",
                        "Email",
                        "email",
                        "email",
                        "you@company.com",
                    ),
                )
                form_status(id: "forgot-password-status")
                submit_button(label: "Send reset link", pending_label: "Creating reset link…")
            </form>
            <p class="mt-6 text-center text-sm text-muted-foreground">
                <a class="font-medium text-foreground underline underline-offset-4" href="/sign-in">
                    "Back to sign in"
                </a>
            </p>
        )
    }
}
