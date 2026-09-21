/// Canonicalizes a project module path with platform-independent separators.
pub(crate) fn canonical(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in normalized.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            value => parts.push(value),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Resolves an import source relative to an importing module path.
pub(crate) fn resolve(module: &str, source: &str) -> Option<String> {
    if source.starts_with('@') {
        return Some(source.into());
    }
    if !source.starts_with('.') {
        return canonical(source);
    }
    let parent = module.rsplit_once('/').map_or("", |(parent, _)| parent);
    canonical(&format!("{parent}/{source}"))
}
