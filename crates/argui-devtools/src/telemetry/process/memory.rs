use crate::telemetry::{MemoryCategory, ProcessTelemetry};
use std::{collections::BTreeMap, io::BufRead};

pub(super) fn enrich(mut metrics: ProcessTelemetry) -> ProcessTelemetry {
    if let Ok(file) = std::fs::File::open("/proc/self/smaps")
        && let Ok(resident) = resident(std::io::BufReader::new(file))
    {
        metrics.resident_bytes = resident.resident_bytes;
        metrics.private_bytes = resident.private_bytes;
        metrics.proportional_bytes = resident.proportional_bytes;
        metrics.categories = resident.categories;
    }
    metrics
}

/// Linux reports resident pages per mapping. Group those exact RSS values;
/// virtual address ranges and GPU allocations are deliberately excluded.
fn resident(mut input: impl BufRead) -> Result<ProcessTelemetry, String> {
    let mut groups = BTreeMap::<&str, BTreeMap<String, u64>>::new();
    let mut mapping = String::new();
    let mut line = String::new();
    let mut total_read = 0;
    let mut rss = 0_u64;
    let mut private = 0;
    let mut pss = 0;
    let mut entries = 0;
    loop {
        line.clear();
        let read = input
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        total_read += read;
        if total_read > 8 * 1024 * 1024 {
            return Err("Memory mappings exceeded the snapshot limit".into());
        }
        let mut fields = line.split_whitespace();
        let Some(first) = fields.next() else {
            continue;
        };
        if first.contains('-') {
            // Address, permissions, offset, device and inode precede the path.
            mapping = fields.skip(4).collect::<Vec<_>>().join(" ");
        } else if matches!(first, "Rss:" | "Pss:" | "Private_Clean:" | "Private_Dirty:") {
            let bytes = fields
                .next()
                .and_then(|value| value.parse::<u64>().ok())
                .and_then(|value| value.checked_mul(1024))
                .ok_or("Invalid resident memory value")?;
            match first {
                "Rss:" => {
                    rss = rss
                        .checked_add(bytes)
                        .ok_or("Resident memory total overflow")?;
                    entries += 1;
                    let category = category(&mapping);
                    let details = groups.entry(category).or_default();
                    let name = if mapping.is_empty() {
                        "Anonymous mappings"
                    } else if details.len() >= 128 && !details.contains_key(&mapping) {
                        "Other mappings"
                    } else {
                        &mapping
                    };
                    *details.entry(name.into()).or_default() += bytes;
                }
                "Pss:" => pss += bytes,
                _ => private += bytes,
            }
        }
    }
    if entries == 0 {
        return Err("No resident mappings reported".into());
    }
    let mut categories: Vec<_> = groups
        .into_iter()
        .map(|(name, details)| {
            let resident_bytes = details.values().sum();
            let mut details: Vec<_> = details
                .into_iter()
                .filter(|(_, bytes)| *bytes > 0)
                .collect();
            details.sort_by_key(|entry| std::cmp::Reverse(entry.1));
            if details.len() > 12 {
                let remainder = details.drain(11..).map(|(_, bytes)| bytes).sum();
                details.push(("Other mappings".into(), remainder));
            }
            MemoryCategory {
                name: name.into(),
                resident_bytes,
                details,
            }
        })
        .collect();
    categories.sort_by_key(|entry| std::cmp::Reverse(entry.resident_bytes));
    Ok(ProcessTelemetry {
        resident_bytes: rss,
        private_bytes: Some(private),
        proportional_bytes: Some(pss),
        categories,
        ..Default::default()
    })
}

fn category(mapping: &str) -> &'static str {
    if mapping == "[heap]" {
        "Heap mapping"
    } else if mapping.starts_with("[stack") {
        "Stack mappings"
    } else if mapping.starts_with("/dev/") || mapping.contains("dmabuf") {
        "Device mappings"
    } else if mapping.is_empty() || mapping.starts_with("[anon") {
        "Anonymous allocations"
    } else if mapping.starts_with('[') {
        "Kernel mappings"
    } else {
        "Files and libraries"
    }
}
