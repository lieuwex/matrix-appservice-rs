use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Generate a `String` of length `n_chars` consisting of cryptographically random alphanumeric
/// characters.
pub fn random_alphanumeric(n_chars: usize) -> String {
    thread_rng()
        .sample_iter(Alphanumeric)
        .take(n_chars)
        .map(char::from)
        .collect()
}
