use topcoat::{
    Result,
    view::{View, attributes, component, view},
};

use super::{brand::brand_link, card::card};

#[component]
pub(crate) async fn auth_panel(
    title: &'static str,
    description: &'static str,
    child: View,
) -> Result {
    view! {
        <main class="grid min-h-screen bg-background lg:grid-cols-[minmax(0,1.05fr)_minmax(28rem,0.95fr)]">
            <section
                class="relative hidden overflow-hidden bg-primary p-12 text-primary-foreground lg:flex lg:flex-col lg:justify-between"
                aria-label="About LaunchLightly"
            >
                <div class="[&_a]:text-primary-foreground">
                    brand_link()
                </div>
                <div class="relative z-10 max-w-lg space-y-5">
                    <p class="text-xs font-semibold tracking-[0.22em] text-primary-foreground/60 uppercase">
                        "Feature delivery infrastructure"
                    </p>
                    <h2 class="text-4xl leading-tight font-semibold tracking-tight">
                        "Gate changes without slowing delivery."
                    </h2>
                    <p class="max-w-md text-base leading-7 text-primary-foreground/70">
                        "Use feature flags to decide what your applications serve, without requiring another deployment."
                    </p>
                </div>
                <p class="relative z-10 text-sm text-primary-foreground/55">
                    "Make the release decision independently from the deploy."
                </p>
                <div
                    class="absolute -right-32 -bottom-32 size-[30rem] rounded-full border border-primary-foreground/10"
                    aria-hidden="true"
                ></div>
                <div
                    class="absolute -right-12 -bottom-12 size-72 rounded-full border border-primary-foreground/10"
                    aria-hidden="true"
                ></div>
            </section>

            <section class="flex items-center justify-center px-5 py-12 sm:px-8">
                <div class="w-full max-w-md space-y-8">
                    <div class="lg:hidden">
                        brand_link()
                    </div>
                    card(
                        attrs: attributes! {
                            class="border-border/80 py-8 shadow-sm"
                        },
                        <div class="space-y-2 px-6 sm:px-8">
                            <h1 class="text-2xl font-semibold tracking-tight">(title)</h1>
                            <p class="text-sm leading-6 text-muted-foreground">(description)</p>
                        </div>
                        <div class="px-6 sm:px-8">(child)</div>
                    )
                </div>
            </section>
        </main>
    }
}
