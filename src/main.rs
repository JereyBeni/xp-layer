//! xp-layer — Windows XP API translation / compatibility layer
//!
//! This is the early skeleton. Future work will add PE loading,
//! API stubs, and host memory detection.

mod config;

fn main() {
    println!("xp-layer — Windows XP compatibility layer (skeleton)");
    println!("No PE loading or API translation has been implemented yet.\n");

    // Demonstrate the memory-reporting policy with a few example host sizes.
    // In a later stage this will use real host detection.
    let examples = [16 * 1024, 8 * 1024, 4 * 1024, 2 * 1024, 1024];
    println!("Memory reporting policy (host MB → reported XP MB):");
    for host_mb in examples {
        let reported = config::reported_xp_memory_mb(host_mb);
        println!("  {:>6} MB host  →  {:>4} MB reported to XP", host_mb, reported);
    }
}
