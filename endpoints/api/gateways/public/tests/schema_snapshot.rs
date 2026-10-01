//! لو السكيما اتغيّرت، التست ده بيقع.
//! لو التغيير مقصود: `UPDATE_SCHEMA=1 cargo test -p gw-public`

const SNAPSHOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/schema/public.graphql");

#[test]
fn schema_matches_snapshot() {
    let sdl = gw_public::schema_sdl();

    if std::env::var("UPDATE_SCHEMA").is_ok() {
        std::fs::write(SNAPSHOT, &sdl).expect("failed to write snapshot");
        return;
    }

    let expected = std::fs::read_to_string(SNAPSHOT)
        .expect("snapshot missing — run with UPDATE_SCHEMA=1");

    assert_eq!(sdl, expected, "سكيما بوابة public اتغيّرت من غير ما تحدّث السناب‑شوت");
}

/// أهم تست في المشروع: البوابة العامة ماتسرّبش حقول حسّاسة.
#[test]
fn public_schema_hides_sensitive_fields() {
    let sdl = gw_public::schema_sdl();

    for leaked in ["salary", "internalNotes", "email", "banned"] {
        assert!(
            !sdl.contains(leaked),
            "البوابة العامة بتعرض حقل مايصحّش: {leaked}"
        );
    }
}
