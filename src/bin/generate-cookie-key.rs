use axum_extra::extract::cookie::Key;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:?}", hex::encode(Key::generate().master()));
    Ok(())
}
