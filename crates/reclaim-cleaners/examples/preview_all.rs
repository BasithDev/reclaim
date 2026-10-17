//! READ-ONLY CLI: prints JSON with info and preview (totalBytes, itemCount, command) of every available cleaner.
//! Never calls clean(). Usage: cargo run -p reclaim-cleaners --example preview_all [-- --items]

use reclaim_cleaners::all_cleaners;
use serde_json::json;

fn main() {
    let with_items = std::env::args().any(|a| a == "--items");
    let mut out = Vec::new();
    for c in all_cleaners() {
        let info = c.info();
        let entry = match c.preview() {
            Ok(p) => {
                let mut v = json!({
                    "id": info.id, "name": info.name, "group": info.group, "risk": info.risk,
                    "requiresAdmin": info.requires_admin, "method": info.method,
                    "totalBytes": p.total_bytes, "itemCount": p.item_count, "command": p.command,
                });
                if with_items {
                    v["items"] = serde_json::to_value(&p.items).unwrap_or_default();
                }
                v
            }
            Err(e) => json!({ "id": info.id, "name": info.name, "error": e.to_string() }),
        };
        out.push(entry);
    }
    println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
}
