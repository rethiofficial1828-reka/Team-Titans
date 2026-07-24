//! Shannon entropy validation module for RansomGuard-style heuristics.

/// Shannon entropy (0.0–8.0). >7.9 ≈ encrypted/compressed.
pub fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
    let mut c = [0u64; 256];
    for &b in data { c[b as usize] += 1; }
    let n = data.len() as f64;
    c.iter().filter(|&&x| x > 0)
        .map(|&x| { let p = x as f64 / n; -p * p.log2() })
        .sum()
}

const MAX_ENTROPY: f64 = 8.0;
const MIN_THRESHOLD: f64 = 6.5;
const ENCRYPTED: f64 = 7.8;

/// RansomGuard-validated encryption test (coefficient 0.83 — statistically
/// tuned against a large corpus to minimize false positives on archives).
pub fn is_encrypted(initial: f64, final_: f64) -> bool {
    if initial <= 0.0 { return false; }
    let diff = final_ - initial;
    let suspicious = (MAX_ENTROPY - initial) * 0.83;   // key insight from RansomGuard
    final_ >= MIN_THRESHOLD && (diff >= suspicious || (initial < ENCRYPTED && final_ >= ENCRYPTED))
}
