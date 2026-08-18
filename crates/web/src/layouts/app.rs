use topcoat::{
    Result,
    context::Cx,
    font::fontsource::fontsource_font,
    router::{layout, request::uri},
    view::view,
};

use crate::assets::{APP_STYLESHEET, AUTH_SCRIPT};

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
                topcoat::font::link(
                    font: fontsource_font!(
                        GEIST,
                        weight: [400, 500, 600, 700],
                        style: Normal,
                        subset: Latin,
                    ),
                )
                <link rel="stylesheet" href=(APP_STYLESHEET)>
            </head>
            <body class="min-h-screen antialiased">
                (slot?)
                <script src=(AUTH_SCRIPT) defer="defer"></script>
                topcoat::dev::script()
            </body>
        </html>
    }
}
