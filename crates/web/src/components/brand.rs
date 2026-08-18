use topcoat::{
    Result,
    view::{component, view},
};

#[component]
pub(crate) async fn brand_link() -> Result {
    view! {
        <a
            class="inline-flex items-center gap-3 text-sm font-semibold tracking-tight text-foreground"
            href="/"
            aria-label="LaunchLightly home"
        >
            <span
                class="relative grid size-9 place-items-center rounded-xl bg-primary text-primary-foreground shadow-xs"
                aria-hidden="true"
            >
                <span class="absolute h-0.5 w-4 -translate-y-1 rotate-12 rounded-full bg-current"></span>
                <span class="absolute h-0.5 w-4 translate-y-1 -rotate-12 rounded-full bg-current"></span>
            </span>
            <span>"LaunchLightly"</span>
        </a>
    }
}
