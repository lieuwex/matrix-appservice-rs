use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Generate a `String` of length `n_chars` consisting of cryptographically random alphanumeric
/// characters.
pub fn random_alphanumeric(n_chars: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|_| rng.sample(Alphanumeric))
        .take(n_chars)
        .collect()
}
