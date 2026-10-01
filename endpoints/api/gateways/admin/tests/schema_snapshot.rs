//! لو السكيما اتغيّرت، التست ده بيقع.
//! لو التغيير مقصود: `UPDATE_SCHEMA=1 cargo test -p gw-admin`

const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/schema/admin.graphql");

#[test]
fn schema_matches_snapshot() {
    let sdl = gw_admin::schema_sdl();

    if std::env::var("UPDATE_SCHEMA").is_ok() {
        std::fs::write(SNAPSHOT, &sdl).expect("failed to write snapshot");
        return;
    }

    let expected = std::fs::read_to_string(SNAPSHOT)
        .expect("snapshot missing — run with UPDATE_SCHEMA=1");

    assert_eq!(sdl, expected, "سكيما بوابة admin اتغيّرت من غير ما تحدّث السناب‑شوت");
}
