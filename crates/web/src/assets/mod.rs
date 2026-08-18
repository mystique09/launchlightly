use topcoat::asset::{Asset, asset};

pub(crate) const APP_STYLESHEET: Asset = topcoat::tailwind::stylesheet!();
pub(crate) const AUTH_SCRIPT: Asset = asset!("assets/auth.js");

#[cfg(test)]
pub(crate) fn test_bundle() -> topcoat::asset::AssetBundle {
    use std::sync::OnceLock;

    use topcoat::asset::{AssetBundle, MANIFEST_NAME, MANIFEST_VERSION, Manifest, ManifestEntry};

    static BUNDLE: OnceLock<AssetBundle> = OnceLock::new();

    BUNDLE
        .get_or_init(|| {
            let directory = std::env::temp_dir().join(format!(
                "launchlightly-web-test-assets-{}",
                std::process::id()
            ));
            std::fs::create_dir_all(&directory).expect("create test asset bundle directory");
            std::fs::write(
                directory.join("app-test.css"),
                include_str!(concat!(env!("OUT_DIR"), "/tailwind.css")),
            )
            .expect("write test stylesheet");
            std::fs::write(
                directory.join("auth-test.js"),
                include_str!("../../assets/auth.js"),
            )
            .expect("write test auth script");
            Manifest {
                version: MANIFEST_VERSION,
                assets: vec![
                    ManifestEntry {
                        id: APP_STYLESHEET.id(),
                        file: "app-test.css".to_owned(),
                        hash: "test".to_owned(),
                        content_type: "text/css; charset=utf-8".to_owned(),
                    },
                    ManifestEntry {
                        id: AUTH_SCRIPT.id(),
                        file: "auth-test.js".to_owned(),
                        hash: "test".to_owned(),
                        content_type: "text/javascript; charset=utf-8".to_owned(),
                    },
                ],
            }
            .save(directory.join(MANIFEST_NAME))
            .expect("write test asset manifest");

            AssetBundle::load_dir(directory).expect("load test asset bundle")
        })
        .clone()
}
