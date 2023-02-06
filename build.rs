fn main() {
    // Retrieve build metadata
    if let Err(err) = shadow_rs::new() {
        eprintln!("Error using shadow_rs to retrieve build metadata: {err:?}");
    }
}