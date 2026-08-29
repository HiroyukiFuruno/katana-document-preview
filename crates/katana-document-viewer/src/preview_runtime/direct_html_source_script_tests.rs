use super::*;

#[test]
fn direct_html_source_uses_krr_v8_to_apply_dom_content_loaded_script() -> Result<(), PreviewError> {
    let output = output_for_html(scripted_html())?;
    let plan = ViewerNodePlanner::create(&output.input, 0.0);

    assert!(
        plan.nodes
            .iter()
            .any(|node| node.text.contains("JavaScript evaluated")),
        "{:#?}",
        plan.nodes
    );
    assert!(
        plan.nodes
            .iter()
            .any(|node| node.text.contains("JS inserted badge")),
        "{:#?}",
        plan.nodes
    );
    assert!(
        !plan
            .nodes
            .iter()
            .any(|node| node.text.contains("JavaScript pending")),
        "{:#?}",
        plan.nodes
    );
    Ok(())
}

fn scripted_html() -> String {
    [
        "<main>",
        r#"<div id="js-status" class="pending">JavaScript pending</div>"#,
        r#"<div id="js-target">JS check</div>"#,
        "<script>",
        r#"document.addEventListener("DOMContentLoaded", () => {"#,
        r#"  const status = document.getElementById("js-status");"#,
        r#"  status.classList.remove("pending");"#,
        r#"  status.textContent = "JavaScript evaluated";"#,
        r#"  const chip = document.createElement("span");"#,
        r#"  chip.className = "badge";"#,
        r#"  chip.textContent = "JS inserted badge";"#,
        r#"  document.getElementById("js-target").appendChild(chip);"#,
        "});",
        "</script>",
        "</main>",
    ]
    .join("\n")
}
