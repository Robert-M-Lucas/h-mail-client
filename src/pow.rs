use rsa::BigUint;

pub fn solve_challenge(challenge: BigUint, n: &BigUint, iters: u64) -> BigUint {
    let mut x = challenge;
    let two = BigUint::from(2usize);
    for _ in 0..iters {
        x = x.modpow(&two, n);
    }

    x
}