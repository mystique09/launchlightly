use topcoat::{Result, router::page, view::view};

use crate::components::{
    auth_panel::auth_panel,
    form_field::{FieldSpec, form_field},
    form_status::form_status,
    submit_button::submit_button,
};

#[page("/sign-up")]
pub(crate) async fn sign_up_page() -> Result {
    view! {
        auth_panel(
            title: "Create your account",
            description: "Start with your name and work email.",
            <form id="sign-up-form" class="flex flex-col gap-5" novalidate="novalidate">
                form_field(
                    field: FieldSpec::new(
                        "name",
                        "Name",
                        "text",
                        "name",
                        "Your name",
                    ).min_length(1),
                )
                form_field(
                    field: FieldSpec::new(
                        "email",
                        "Email",
                        "email",
                        "email",
                        "you@company.com",
                    ),
                )
                form_field(
                    field: FieldSpec::new(
                        "password",
                        "Password",
                        "password",
                        "new-password",
                        "Create a password",
                    ).hint("8–128 characters").min_length(8),
                )
                form_field(
                    field: FieldSpec::new(
                        "confirm-password",
                        "Confirm password",
                        "password",
                        "new-password",
                        "Repeat your password",
                    ).min_length(8),
                )
                form_status(id: "sign-up-status")
                submit_button(label: "Create account", pending_label: "Creating account…")
            </form>
            <p class="mt-6 text-center text-sm text-muted-foreground">
                "Already have an account? "
                <a class="font-medium text-foreground underline underline-offset-4" href="/sign-in">
                    "Sign in"
                </a>
            </p>
        )
    }
}
