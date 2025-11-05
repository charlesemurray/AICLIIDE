/// Generate a nice human-readable session name
pub fn generate_session_name() -> String {
    use rand::Rng;
    
    let adjectives = [
        "swift", "bright", "calm", "bold", "clever", "eager", "gentle", "happy",
        "keen", "lively", "merry", "noble", "proud", "quick", "wise", "brave",
        "cool", "fair", "kind", "neat", "pure", "safe", "warm", "wild",
    ];
    
    let nouns = [
        "fox", "owl", "hawk", "wolf", "bear", "lion", "tiger", "eagle",
        "falcon", "raven", "swan", "deer", "otter", "lynx", "puma", "cobra",
        "shark", "whale", "dolphin", "seal", "orca", "ray", "coral", "pearl",
    ];
    
    let mut rng = rand::thread_rng();
    let adj = adjectives[rng.gen_range(0..adjectives.len())];
    let noun = nouns[rng.gen_range(0..nouns.len())];
    
    format!("{}-{}", adj, noun)
}
