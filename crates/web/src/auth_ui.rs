use topcoat::{
    Result,
    context::Cx,
    router::{layout, page, request::uri},
    view::{View, component, view},
};

const AUTH_CSS: &str = include_str!("auth.css");
const AUTH_JS: &str = include_str!("auth.js");

#[layout("/")]
pub(crate) async fn app_layout(cx: &Cx, slot: Result) -> Result {
    let title = match uri(cx).path() {
        "/sign-in" => "Sign in · LaunchLightly",
        "/sign-up" => "Create account · LaunchLightly",
        "/account/security" => "Account security · LaunchLightly",
        _ => "LaunchLightly",
    };

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="color-scheme" content="light">
                <title>(title)</title>
                <link
                    rel="icon"
                    href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='9' fill='%2317221b'/%3E%3Cpath d='M9 11l14 5-14 5' fill='none' stroke='%23fff' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E"
                >
                <style>(View::unescaped_unchecked(AUTH_CSS))</style>
            </head>
            <body>
                (slot?)
                <script>(View::unescaped_unchecked(AUTH_JS))</script>
            </body>
        </html>
    }
}

#[component]
async fn brand_link() -> Result {
    view! {
        <a class="brand" href="/" aria-label="LaunchLightly home">
            <span class="brand-mark" aria-hidden="true">
                <span></span>
                <span></span>
            </span>
            <span>"LaunchLightly"</span>
        </a>
    }
}

#[component]
async fn form_field(field: FieldSpec) -> Result {
    let hint_id = format!("{}-hint", field.id);

    view! {
        <div class="field">
            <div class="field-heading">
                <label for=(field.id)>(field.label)</label>
                if !field.hint.is_empty() {
                    <span id=(&hint_id)>(field.hint)</span>
                }
            </div>
            <input
                id=(field.id)
                name=(field.id)
                type=(field.input_type)
                autocomplete=(field.autocomplete)
                placeholder=(field.placeholder)
                required="required"
                if !field.hint.is_empty() {
                    aria-describedby=(&hint_id)
                }
                if let Some(min_length) = field.min_length {
                    minlength=(min_length)
                }
                if field.input_type == "password" {
                    maxlength="128"
                }
            >
        </div>
    }
}

struct FieldSpec {
    id: &'static str,
    label: &'static str,
    input_type: &'static str,
    autocomplete: &'static str,
    placeholder: &'static str,
    hint: &'static str,
    min_length: Option<usize>,
}

#[component]
async fn submit_button(label: &str, pending_label: &str) -> Result {
    view! {
        <button
            class="button button-primary"
            type="submit"
            data-default-label=(label)
            data-pending-label=(pending_label)
        >
            <span data-button-label="true">(label)</span>
        </button>
    }
}

#[component]
async fn form_status(id: &str) -> Result {
    view! {
        <p
            id=(id)
            class="form-status"
            role="alert"
            aria-live="polite"
            tabindex="-1"
            hidden="hidden"
        ></p>
    }
}

#[component]
async fn auth_panel(title: &str, description: &str, child: View) -> Result {
    view! {
        <main class="auth-scene">
            <section class="product-context" aria-label="About LaunchLightly">
                brand_link()
                <div class="product-message">
                    <h2>"Gate changes without slowing delivery."</h2>
                    <p>
                        "Use feature flags to decide what your applications serve, without requiring another deployment."
                    </p>
                </div>
                <p class="context-note">"Make the release decision independently from the deploy."</p>
            </section>

            <section class="auth-panel">
                <div class="auth-heading">
                    <h1>(title)</h1>
                    <p>(description)</p>
                </div>
                (child)
            </section>
        </main>
    }
}

#[page("/")]
pub(crate) async fn index_page() -> Result {
    view! {
        <main
            class="session-gate"
            data-session-gate="true"
            data-session-endpoint="/api/auth/get-session"
            aria-live="polite"
        >
            <div class="gate-mark" aria-hidden="true">
                <span></span>
                <span></span>
            </div>
            <p id="session-gate-status">"Opening LaunchLightly…"</p>
            <a id="session-gate-retry" href="/" hidden="hidden">"Try again"</a>
            <noscript>
                <p>"JavaScript is required to check your session. " <a href="/sign-in">"Continue to sign in"</a> "."</p>
            </noscript>
        </main>
    }
}

#[page("/sign-in")]
pub(crate) async fn sign_in_page() -> Result {
    view! {
        auth_panel(
            title: "Sign in to LaunchLightly",
            description: "Use the email and password attached to your account.",
            <form id="sign-in-form" class="auth-form" novalidate="novalidate">
                form_field(
                    field: FieldSpec {
                        id: "email",
                        label: "Email",
                        input_type: "email",
                        autocomplete: "email",
                        placeholder: "you@company.com",
                        hint: "",
                        min_length: None,
                    },
                )
                form_field(
                    field: FieldSpec {
                        id: "password",
                        label: "Password",
                        input_type: "password",
                        autocomplete: "current-password",
                        placeholder: "Your password",
                        hint: "",
                        min_length: Some(8),
                    },
                )
                form_status(id: "sign-in-status")
                submit_button(label: "Sign in", pending_label: "Signing in…")
            </form>
            <p class="auth-switch">
                "New to LaunchLightly? " <a href="/sign-up">"Create an account"</a>
            </p>
        )
    }
}

#[page("/sign-up")]
pub(crate) async fn sign_up_page() -> Result {
    view! {
        auth_panel(
            title: "Create your account",
            description: "Start with your name and work email.",
            <form id="sign-up-form" class="auth-form" novalidate="novalidate">
                form_field(
                    field: FieldSpec {
                        id: "name",
                        label: "Name",
                        input_type: "text",
                        autocomplete: "name",
                        placeholder: "Your name",
                        hint: "",
                        min_length: Some(1),
                    },
                )
                form_field(
                    field: FieldSpec {
                        id: "email",
                        label: "Email",
                        input_type: "email",
                        autocomplete: "email",
                        placeholder: "you@company.com",
                        hint: "",
                        min_length: None,
                    },
                )
                form_field(
                    field: FieldSpec {
                        id: "password",
                        label: "Password",
                        input_type: "password",
                        autocomplete: "new-password",
                        placeholder: "Create a password",
                        hint: "8–128 characters",
                        min_length: Some(8),
                    },
                )
                form_field(
                    field: FieldSpec {
                        id: "confirm-password",
                        label: "Confirm password",
                        input_type: "password",
                        autocomplete: "new-password",
                        placeholder: "Repeat your password",
                        hint: "",
                        min_length: Some(8),
                    },
                )
                form_status(id: "sign-up-status")
                submit_button(label: "Create account", pending_label: "Creating account…")
            </form>
            <p class="auth-switch">
                "Already have an account? " <a href="/sign-in">"Sign in"</a>
            </p>
        )
    }
}

#[page("/account/security")]
pub(crate) async fn security_page() -> Result {
    view! {
        <main class="security-page" data-account-page="true">
            <header class="security-header">
                brand_link()
                <div class="security-header-action">
                    <p
                        id="sign-out-status"
                        class="form-status"
                        role="alert"
                        aria-live="polite"
                        tabindex="-1"
                        hidden="hidden"
                    ></p>
                    <button id="sign-out" class="button button-quiet" type="button">"Sign out"</button>
                </div>
            </header>

            <div id="account-loading" class="account-loading" aria-live="polite">
                <span class="loading-line loading-line-long"></span>
                <span class="loading-line"></span>
                <span class="sr-only">"Loading account security"</span>
            </div>

            <div id="account-content" class="security-surface" hidden="hidden">
                <header class="security-title">
                    <div>
                        <h1>"Account security"</h1>
                        <p>"Manage your password and signed-in sessions."</p>
                    </div>
                    <div class="identity" aria-label="Signed-in account">
                        <span id="account-name"></span>
                        <span id="account-email"></span>
                    </div>
                </header>

                <section class="security-section" aria-labelledby="password-heading">
                    <div class="section-copy">
                        <h2 id="password-heading">"Change password"</h2>
                        <p>"Use at least 8 characters. You can sign out every other session at the same time."</p>
                    </div>
                    <form id="change-password-form" class="security-form" novalidate="novalidate">
                        <label class="visually-hidden" for="account-username">"Account email"</label>
                        <input
                            id="account-username"
                            class="visually-hidden"
                            name="username"
                            type="email"
                            autocomplete="username"
                            tabindex="-1"
                        >
                        form_field(
                            field: FieldSpec {
                                id: "current-password",
                                label: "Current password",
                                input_type: "password",
                                autocomplete: "current-password",
                                placeholder: "Your current password",
                                hint: "",
                                min_length: Some(8),
                            },
                        )
                        form_field(
                            field: FieldSpec {
                                id: "new-password",
                                label: "New password",
                                input_type: "password",
                                autocomplete: "new-password",
                                placeholder: "Your new password",
                                hint: "8–128 characters; some symbols count as more than one",
                                min_length: Some(8),
                            },
                        )
                        form_field(
                            field: FieldSpec {
                                id: "confirm-new-password",
                                label: "Confirm new password",
                                input_type: "password",
                                autocomplete: "new-password",
                                placeholder: "Repeat your new password",
                                hint: "",
                                min_length: Some(8),
                            },
                        )
                        <label class="check-row" for="revoke-on-change">
                            <input id="revoke-on-change" name="revoke-on-change" type="checkbox">
                            <span>
                                <strong>"Sign out other sessions"</strong>
                                <small>"Keep only this browser signed in after the password changes."</small>
                            </span>
                        </label>
                        form_status(id: "change-password-status")
                        submit_button(label: "Update password", pending_label: "Updating password…")
                    </form>
                </section>

                <section class="security-section" aria-labelledby="sessions-heading">
                    <div class="section-copy">
                        <h2 id="sessions-heading">"Active sessions"</h2>
                        <p>"Review when your sessions began and when they expire. Session secrets are never displayed."</p>
                    </div>
                    <div class="sessions-panel">
                        <ol id="sessions-list" class="sessions-list" aria-live="polite"></ol>
                        <div class="session-actions">
                            <p id="sessions-status" class="form-status" role="status" aria-live="polite" tabindex="-1" hidden="hidden"></p>
                            <button id="revoke-other-sessions" class="button button-secondary" type="button">
                                "Sign out other sessions"
                            </button>
                        </div>
                    </div>
                </section>
            </div>
        </main>
    }
}
