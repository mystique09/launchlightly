use topcoat::{Result, router::page, view::view};

use crate::components::button::{ButtonSize, ButtonVariant, button_variants};

#[page("/")]
pub(crate) async fn index_page() -> Result {
    view! {
        <main
            class="grid min-h-screen place-items-center bg-background px-6"
            data-session-gate="true"
            data-session-endpoint="/api/auth/get-session"
            data-authenticated-destination="/account/security"
            data-unauthenticated-destination="/sign-in"
            aria-live="polite"
        >
            <div class="flex max-w-sm flex-col items-center gap-5 text-center">
                <div class="relative size-12" aria-hidden="true">
                    <span class="absolute inset-0 animate-ping rounded-full bg-primary/10"></span>
                    <span class="relative grid size-12 place-items-center rounded-2xl bg-primary text-primary-foreground shadow-sm">
                        <span class="absolute h-0.5 w-5 -translate-y-1.5 rotate-12 rounded-full bg-current"></span>
                        <span class="absolute h-0.5 w-5 translate-y-1.5 -rotate-12 rounded-full bg-current"></span>
                    </span>
                </div>
                <p id="session-gate-status" class="text-sm text-muted-foreground">
                    "Opening LaunchLightly…"
                </p>
                <a
                    id="session-gate-retry"
                    class=(button_variants(ButtonVariant::Outline, ButtonSize::Md))
                    href="/"
                    hidden="hidden"
                >
                    "Try again"
                </a>
                <noscript>
                    <p class="text-sm text-muted-foreground">
                        "JavaScript is required to check your session. "
                        <a class="font-medium text-foreground underline underline-offset-4" href="/sign-in">
                            "Continue to sign in"
                        </a>
                        "."
                    </p>
                </noscript>
            </div>
        </main>
    }
}
