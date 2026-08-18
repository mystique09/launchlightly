fn main() {
    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("feather")
        .stage()
        .expect("stage Feather icons");

    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .expect("compile Tailwind stylesheet");
}
