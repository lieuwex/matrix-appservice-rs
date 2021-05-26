/// Generate a `String` of length `n_chars` consisting of cryptographically random alphanumeric
/// characters.
#[cfg(feature = "rand")]
pub fn random_alphanumeric(n_chars: usize) -> String {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(n_chars)
        .map(char::from)
        .collect()
}
