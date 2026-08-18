use topcoat::{
    Result,
    router::page,
    view::{attributes, view},
};

use crate::components::{
    brand::brand_link,
    button::{ButtonSize, ButtonVariant, button},
    card::card,
    checkbox::checkbox,
    form_field::{FieldSpec, form_field},
    form_status::form_status,
    label::label,
    skeleton::skeleton,
    submit_button::submit_button,
};

#[page("/account/security")]
pub(crate) async fn account_security_page() -> Result {
    view! {
        <main class="min-h-screen bg-foreground/[0.025]" data-account-page="true">
            <header class="border-b border-border bg-background">
                <div class="mx-auto flex max-w-6xl items-center justify-between gap-6 px-5 py-4 sm:px-8">
                    brand_link()
                    <div class="flex items-center gap-3">
                        form_status(id: "sign-out-status")
                        button(
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Sm,
                            attrs: attributes! { id="sign-out" type="button" },
                            "Sign out"
                        )
                    </div>
                </div>
            </header>

            <div class="mx-auto max-w-6xl px-5 py-10 sm:px-8 sm:py-14">
                <div id="account-loading" class="space-y-5" aria-live="polite">
                    skeleton(attrs: attributes! { class="h-8 w-64 rounded-lg" })
                    skeleton(attrs: attributes! { class="h-4 w-96 max-w-full" })
                    <span class="sr-only">"Loading account security"</span>
                </div>

                <div id="account-content" class="space-y-8" hidden="hidden">
                    <header class="flex flex-col justify-between gap-5 border-b border-border pb-8 sm:flex-row sm:items-end">
                        <div class="space-y-2">
                            <p class="text-xs font-semibold tracking-[0.18em] text-muted-foreground uppercase">
                                "Account"
                            </p>
                            <h1 class="text-3xl font-semibold tracking-tight">"Account security"</h1>
                            <p class="text-sm text-muted-foreground">
                                "Manage your password and signed-in sessions."
                            </p>
                        </div>
                        <div class="flex flex-col text-sm sm:items-end" aria-label="Signed-in account">
                            <span id="account-name" class="font-medium"></span>
                            <span id="account-email" class="text-muted-foreground"></span>
                        </div>
                    </header>

                    <div class="grid items-start gap-6 lg:grid-cols-2">
                        <section aria-labelledby="password-heading">
                            card(
                                attrs: attributes! { class="py-7" },
                                <div class="space-y-2 px-6">
                                    <h2 id="password-heading" class="text-lg font-semibold">"Change password"</h2>
                                    <p class="text-sm leading-6 text-muted-foreground">
                                        "Use at least 8 characters. You can sign out every other session at the same time."
                                    </p>
                                </div>
                                <form
                                    id="change-password-form"
                                    class="flex flex-col gap-5 border-t border-border px-6 pt-6"
                                    novalidate="novalidate"
                                >
                                    <label class="sr-only" for="account-username">"Account email"</label>
                                    <input
                                        id="account-username"
                                        class="sr-only"
                                        name="username"
                                        type="email"
                                        autocomplete="username"
                                        tabindex="-1"
                                    >
                                    form_field(
                                        field: FieldSpec::new(
                                            "current-password",
                                            "Current password",
                                            "password",
                                            "current-password",
                                            "Your current password",
                                        ).min_length(8),
                                    )
                                    form_field(
                                        field: FieldSpec::new(
                                            "new-password",
                                            "New password",
                                            "password",
                                            "new-password",
                                            "Your new password",
                                        ).hint("8–128 characters; some symbols count as more than one").min_length(8),
                                    )
                                    form_field(
                                        field: FieldSpec::new(
                                            "confirm-new-password",
                                            "Confirm new password",
                                            "password",
                                            "new-password",
                                            "Repeat your new password",
                                        ).min_length(8),
                                    )
                                    label(
                                        attrs: attributes! {
                                            class="items-start rounded-lg border border-border p-3 leading-normal"
                                            for="revoke-on-change"
                                        },
                                        checkbox(
                                            attrs: attributes! {
                                                id="revoke-on-change"
                                                class="mt-0.5"
                                                name="revoke-on-change"
                                            },
                                        )
                                        <span class="flex flex-col gap-0.5">
                                            <strong class="text-sm font-medium">"Sign out other sessions"</strong>
                                            <small class="text-xs leading-5 text-muted-foreground">
                                                "Keep only this browser signed in after the password changes."
                                            </small>
                                        </span>
                                    )
                                    form_status(id: "change-password-status")
                                    submit_button(label: "Update password", pending_label: "Updating password…")
                                </form>
                            )
                        </section>

                        <section aria-labelledby="sessions-heading">
                            card(
                                attrs: attributes! { class="py-7" },
                                <div class="space-y-2 px-6">
                                    <h2 id="sessions-heading" class="text-lg font-semibold">"Active sessions"</h2>
                                    <p class="text-sm leading-6 text-muted-foreground">
                                        "Review when your sessions began and when they expire. Session secrets are never displayed."
                                    </p>
                                </div>
                                <div class="border-t border-border px-6 pt-6">
                                    <ol id="sessions-list" class="space-y-3" aria-live="polite"></ol>
                                    <div class="mt-5 space-y-3 border-t border-border pt-5">
                                        form_status(id: "sessions-status", status: true)
                                        button(
                                            variant: ButtonVariant::Outline,
                                            attrs: attributes! {
                                                id="revoke-other-sessions"
                                                class="w-full"
                                                type="button"
                                            },
                                            "Sign out other sessions"
                                        )
                                    </div>
                                </div>
                            )
                        </section>
                    </div>
                </div>
            </div>
        </main>
    }
}
