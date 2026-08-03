use crate::preview_runtime::asset_loader_cache::PreviewDiagramAssetCache;
use crate::{DiagramRenderCacheOptions, KdvThemeSnapshot};
use katana_markdown_model::DiagramKind;

#[path = "asset_loader_cache_tests_disk_cache_helpers.rs"]
mod helpers;
use helpers::{clean_cache_root, count_svg_files, locate_svg_file, output_for, unique_cache_root};
#[cfg(unix)]
use helpers::{disk_cache_key_and_permissions, restore_permissions, set_readonly};

#[test]
fn diagram_disk_cache_discard_non_svg_payload() -> Result<(), Box<dyn std::error::Error>> {
    let root = unique_cache_root("rejected-svg");
    let output = output_for("```mermaid\ngraph TD\n  PlainSvgA --> PlainSvgB\n```")?;
    let node = output.input.snapshot.document.nodes[0].clone();
    let key = PreviewDiagramAssetCache::key(
        "engine",
        &output,
        &node,
        &DiagramKind::Mermaid,
        "A --> B",
        &KdvThemeSnapshot::katana_light(),
        &DiagramRenderCacheOptions::default(),
    );

    PreviewDiagramAssetCache::put_disk(&root, &key, b"not svg bytes");
    assert_eq!(0, count_svg_files(&root));
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn diagram_disk_cache_removes_invalid_svg_file_content() -> Result<(), Box<dyn std::error::Error>> {
    let root = unique_cache_root("invalid-svg");
    let output = output_for("```mermaid\ngraph TD\n  InvalidSvgA --> InvalidSvgB\n```")?;
    let node = output.input.snapshot.document.nodes[0].clone();
    let options = DiagramRenderCacheOptions::default();
    let key = PreviewDiagramAssetCache::key(
        "engine",
        &output,
        &node,
        &DiagramKind::Mermaid,
        "A --> B",
        &KdvThemeSnapshot::katana_light(),
        &options,
    );

    PreviewDiagramAssetCache::put_disk(&root, &key, b"<svg><text>before</text></svg>");
    let path = locate_svg_file(&root).ok_or("missing svg file")?;
    std::fs::write(&path, [0xff, 0xfe, 0xfd])?;

    assert!(PreviewDiagramAssetCache::get_disk(&root, &key).is_none());
    assert!(!path.exists());
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn diagram_disk_cache_put_disk_ignores_uncreateable_cache_root()
-> Result<(), Box<dyn std::error::Error>> {
    let root = unique_cache_root("unwriteable-root");
    std::fs::write(&root, b"reserved")?;
    let output = output_for("```mermaid\ngraph TD\n  RejectedRootA --> RejectedRootB\n```")?;
    let node = output.input.snapshot.document.nodes[0].clone();
    let key = PreviewDiagramAssetCache::key(
        "engine",
        &output,
        &node,
        &DiagramKind::Mermaid,
        "A --> B",
        &KdvThemeSnapshot::katana_light(),
        &DiagramRenderCacheOptions::default(),
    );

    PreviewDiagramAssetCache::put_disk(&root, &key, b"<svg><text>blocked</text></svg>");

    assert_eq!(0, count_svg_files(&root));
    std::fs::remove_file(&root).ok();
    Ok(())
}

#[cfg(unix)]
#[test]
fn diagram_disk_cache_put_disk_ignores_root_file_write_failure()
-> Result<(), Box<dyn std::error::Error>> {
    let root = unique_cache_root("readonly-parent");
    let (cache_path, original_permissions) = disk_cache_key_and_permissions(&root)?;
    let output = output_for("```mermaid\ngraph TD\n  ReadonlyParentA --> ReadonlyParentB\n```")?;
    let node = output.input.snapshot.document.nodes[0].clone();
    let key = PreviewDiagramAssetCache::key(
        "engine",
        &output,
        &node,
        &DiagramKind::Mermaid,
        "A --> B",
        &KdvThemeSnapshot::katana_light(),
        &DiagramRenderCacheOptions::default(),
    );

    let Some(parent) = cache_path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent)?;
    set_readonly(parent, &original_permissions)?;
    PreviewDiagramAssetCache::put_disk(&root, &key, b"<svg><text>readonly</text></svg>");

    assert_eq!(0, count_svg_files(&root));
    restore_permissions(parent, original_permissions)?;
    clean_cache_root(&root)?;
    Ok(())
}
