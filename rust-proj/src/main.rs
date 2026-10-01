mod utils;

use utils::{config, Claims, create_token, hash_password, verify_password, verify_token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // حمّل ملف .env قبل أي قراءة من البيئة
    config::load();

    let jwt_secret = config::jwt_secret()?;
    let jwt_ttl = config::jwt_ttl_seconds()?;
    // =========================
    // REGISTER
    // =========================

    let password = "my-password";

    let password_hash = hash_password(password)?;

    println!("Password hash:");
    println!("{}", password_hash);

    // في التطبيق الحقيقي:
    // password_hash -> Database

    // =========================
    // LOGIN
    // =========================

    let login_password = "my-password";

    if !verify_password(login_password, &password_hash)? {
        println!("Wrong password!");
        return Ok(());
    }

    println!("Password is correct!");

    // =========================
    // CREATE JWT
    // =========================

    let token = create_token("123", jwt_secret.as_bytes(), jwt_ttl)?;

    println!("\nJWT:");
    println!("{}", token);

    // =========================
    // VERIFY JWT
    // =========================

    let claims: Claims = verify_token(&token, jwt_secret.as_bytes())?;

    println!("\nToken claims:");
    println!("{:?}", claims);

    Ok(())
}
