use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

pub fn random_alphanumeric(n_chars: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat(())
        .map(|_| rng.sample(Alphanumeric))
        .take(n_chars)
        .collect()
}
