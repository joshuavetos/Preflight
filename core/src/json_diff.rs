// A diff engine that produces a deterministic JSON structure for the dashboard.

use serde_json::{json, Value};

/// Return a JSON diff:
/// {
///   "added": {...},
///   "removed": {...},
///   "changed": { key: { "from": ..., "to": ... } }
/// }
pub fn diff_states(current: &Value, proposed: &Value) -> Value {
    match (current, proposed) {
        (Value::Object(a), Value::Object(b)) => {
            let mut added = serde_json::Map::new();
            let mut removed = serde_json::Map::new();
            let mut changed = serde_json::Map::new();

            for (k, v) in b.iter() {
                if !a.contains_key(k) {
                    added.insert(k.clone(), v.clone());
                }
            }

            for (k, v) in a.iter() {
                if !b.contains_key(k) {
                    removed.insert(k.clone(), v.clone());
                }
            }

            for (k, v1) in a.iter() {
                if let Some(v2) = b.get(k) {
                    if v1 != v2 {
                        changed.insert(k.clone(), json!({ "from": v1.clone(), "to": v2.clone() }));
                    }
                }
            }

            json!({
                "added": added,
                "removed": removed,
                "changed": changed,
            })
        }

        // Fallback: if not objects, check raw equality
        (a, b) if a != b => json!({
            "changed": { "from": a.clone(), "to": b.clone() }
        }),

        _ => json!({}),
    }
}
