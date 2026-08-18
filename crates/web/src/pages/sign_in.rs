use topcoat::{Result, router::page, view::view};

use crate::components::{
    auth_panel::auth_panel,
    form_field::{FieldSpec, form_field},
    form_status::form_status,
    submit_button::submit_button,
};

#[page("/sign-in")]
pub(crate) async fn sign_in_page() -> Result {
    view! {
        auth_panel(
            title: "Sign in to LaunchLightly",
            description: "Use the email and password attached to your account.",
            <form id="sign-in-form" class="flex flex-col gap-5" novalidate="novalidate">
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
                        "current-password",
                        "Your password",
                    ).min_length(8),
                )
                <div class="-mt-2 text-right text-sm">
                    <a class="font-medium text-foreground underline underline-offset-4" href="/forgot-password">
                        "Forgot password?"
                    </a>
                </div>
                form_status(id: "sign-in-status")
                submit_button(label: "Sign in", pending_label: "Signing in…")
            </form>
            <p class="mt-6 text-center text-sm text-muted-foreground">
                "New to LaunchLightly? "
                <a class="font-medium text-foreground underline underline-offset-4" href="/sign-up">
                    "Create an account"
                </a>
            </p>
        )
    }
}
