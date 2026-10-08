use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn readme_image_targets(readme: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = readme;
    while let Some(start) = rest.find("![") {
        rest = &rest[start + 2..];
        let Some(mid) = rest.find("](") else { break };
        rest = &rest[mid + 2..];
        let Some(end) = rest.find(')') else { break };
        let target = rest[..end].trim().to_string();
        rest = &rest[end + 1..];
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
            || target.starts_with('#')
            || target.is_empty()
        {
            continue;
        }
        let target = target.split_whitespace().next().unwrap_or("").to_string();
        if !target.is_empty() {
            targets.push(target);
        }
    }
    targets
}

#[test]
fn readme_screenshot_lives_under_public() {
    let root = repo_root();
    let readme =
        std::fs::read_to_string(root.join("README.md")).expect("README.md should be readable");

    assert!(
        readme.contains("public/screenshot.png"),
        "README.md should reference public/screenshot.png"
    );
    assert!(
        root.join("public/screenshot.png").is_file(),
        "public/screenshot.png should exist"
    );
    assert!(
        !root.join("screenshot.png").exists(),
        "legacy screenshot.png at repo root should be gone"
    );
}

#[test]
fn readme_local_images_resolve_on_disk() {
    let root = repo_root();
    let readme =
        std::fs::read_to_string(root.join("README.md")).expect("README.md should be readable");

    for target in readme_image_targets(&readme) {
        let path = root.join(&target);
        assert!(
            path.is_file(),
            "README image target '{target}' should exist on disk"
        );
    }
}
