use super::*;

fn make_dom_element(
    tag: &str,
    class: &str,
    text: &str,
    parent_tag: Option<&str>,
    sibling_index: usize,
    depth: u8,
) -> DomElement {
    let mut attrs = std::collections::HashMap::new();
    if !class.is_empty() {
        attrs.insert("class".to_string(), class.to_string());
    }
    DomElement {
        tag: tag.to_string(),
        attributes: attrs,
        text: text.to_string(),
        parent_tag: parent_tag.map(|s| s.to_string()),
        parent_classes: Vec::new(),
        sibling_index,
        depth,
    }
}

fn make_fingerprint(
    tag: &str,
    class: &str,
    text: &str,
    parent_tag: Option<&str>,
    sibling_index: usize,
    depth: u8,
) -> ElementFingerprint {
    let mut class_names: Vec<String> = if class.is_empty() {
        Vec::new()
    } else {
        class.split_whitespace().map(|s| s.to_string()).collect()
    };
    class_names.sort();

    ElementFingerprint {
        tag: tag.to_string(),
        class_names,
        text_content: text.trim().to_string(),
        href: None,
        src: None,
        data_attrs: std::collections::HashMap::new(),
        parent_tag: parent_tag.map(|s| s.to_string()),
        parent_classes: Vec::new(),
        sibling_index,
        depth,
    }
}

#[test]
fn test_element_fingerprint_creation() {
    let fp = make_fingerprint("div", "container main", "hello world", Some("body"), 0, 2);
    assert_eq!(fp.tag, "div");
    assert_eq!(fp.class_names, vec!["container", "main"]);
    assert_eq!(fp.text_content, "hello world");
    assert_eq!(fp.parent_tag, Some("body".to_string()));
    assert_eq!(fp.sibling_index, 0);
    assert_eq!(fp.depth, 2);
}

#[test]
fn test_save_element_and_count() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("a", "link", "Click", None, 0, 1);
    tracker.save_element("my_link", "a.link", fp);
    assert_eq!(tracker.saved.len(), 1);
    assert_eq!(tracker.saved[0].name, "my_link");
    assert_eq!(tracker.saved[0].original_selector, "a.link");
}

#[test]
fn test_locate_exact_match() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "content", "Hello World", Some("main"), 1, 2);
    tracker.save_element("content_div", "div.content", fp);

    let dom = DomSnapshot::new(vec![
        make_dom_element("header", "nav", "Menu", Some("body"), 0, 1),
        make_dom_element("div", "content", "Hello World", Some("main"), 1, 2),
        make_dom_element("footer", "foot", "End", Some("body"), 2, 1),
    ]);

    let result = tracker.locate("content_div", &dom);
    assert!(result.is_some());
    let m = result.expect("unexpected None/Err");
    assert!(m.similarity > 0.9);
    assert!(m.selector.contains("div") || m.selector.contains("content"));
}

#[test]
fn test_locate_fuzzy_match_modified_element() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "content", "Hello World", Some("main"), 1, 2);
    tracker.save_element("content_div", "div.content", fp);

    let dom = DomSnapshot::new(vec![make_dom_element(
        "div",
        "content-wrapper",
        "Hello World!",
        Some("main"),
        1,
        2,
    )]);

    let result = tracker.locate("content_div", &dom);
    assert!(result.is_some());
    assert!(result.expect("unexpected None/Err").similarity >= 0.6);
}

#[test]
fn test_locate_no_match() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "content", "Hello World", Some("main"), 1, 2);
    tracker.save_element("content_div", "div.content", fp);

    let dom = DomSnapshot::new(vec![
        make_dom_element("span", "icon", "X", None, 0, 1),
        make_dom_element("button", "btn", "Submit", None, 1, 1),
    ]);

    let result = tracker.locate("content_div", &dom);
    assert!(result.is_none());
}

#[test]
fn test_find_similar_exact_match_top() {
    let tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "card", "Product Card", Some("section"), 2, 3);

    let dom = DomSnapshot::new(vec![
        make_dom_element("div", "card", "Product Card", Some("section"), 2, 3),
        make_dom_element("div", "footer", "Footer", Some("body"), 0, 1),
    ]);

    let results = tracker.find_similar(&fp, &dom, 0.6);
    assert!(!results.is_empty());
    assert!((results[0].similarity - 1.0).abs() < 0.01);
}

#[test]
fn test_find_similar_partial_match_ranked() {
    let tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "card highlight", "Special", Some("main"), 1, 2);

    let dom = DomSnapshot::new(vec![
        make_dom_element("div", "card", "Special Item", Some("main"), 2, 2),
        make_dom_element("div", "highlight", "Special Offer", Some("main"), 1, 2),
        make_dom_element("span", "tag", "Other", None, 0, 1),
    ]);

    let results = tracker.find_similar(&fp, &dom, 0.3);
    assert!(results.len() >= 2);
    for pair in results.windows(2) {
        assert!(pair[0].similarity >= pair[1].similarity);
    }
}

#[test]
fn test_similarity_identical() {
    let a = make_fingerprint("div", "main content", "Hello", Some("body"), 1, 2);
    let b = make_fingerprint("div", "main content", "Hello", Some("body"), 1, 2);
    let sim = AdaptiveTracker::similarity(&a, &b);
    assert!((sim - 1.0).abs() < 0.01);
}

#[test]
fn test_similarity_completely_different() {
    let a = make_fingerprint("div", "main", "Hello", Some("body"), 0, 1);
    let b = make_fingerprint("span", "icon", "X", Some("header"), 5, 4);
    let sim = AdaptiveTracker::similarity(&a, &b);
    assert!(sim < 0.3);
}

#[test]
fn test_similarity_same_tag_diff_classes() {
    let a = make_fingerprint("div", "main content", "Hello", Some("body"), 1, 2);
    let b = make_fingerprint("div", "sidebar footer", "World", Some("body"), 2, 2);
    let sim = AdaptiveTracker::similarity(&a, &b);
    let tag_weight = 0.3;
    assert!(sim >= tag_weight * 0.99 && sim < 0.9);
}

#[test]
fn test_similarity_with_data_attributes() {
    let mut a = make_fingerprint("div", "card", "Item", Some("main"), 0, 2);
    a.data_attrs
        .insert("data-id".to_string(), "123".to_string());
    a.data_attrs
        .insert("data-type".to_string(), "product".to_string());

    let mut b = make_fingerprint("div", "card", "Item", Some("main"), 0, 2);
    b.data_attrs
        .insert("data-id".to_string(), "456".to_string());
    b.data_attrs
        .insert("data-type".to_string(), "product".to_string());

    let sim = AdaptiveTracker::similarity(&a, &b);
    let data_weight = 0.15;
    let min_data_score = data_weight * 0.5;
    assert!(sim > 0.8);
    assert!(sim >= 0.3 + 0.2 + 0.15 + min_data_score + 0.1 + 0.1);
}

#[test]
fn test_similarity_parent_context() {
    let mut a = make_fingerprint("div", "item", "Content", Some("section"), 0, 2);
    a.parent_classes = vec!["wrapper".to_string(), "main".to_string()];

    let mut b = make_fingerprint("div", "item", "Content", Some("section"), 0, 2);
    b.parent_classes = vec!["wrapper".to_string(), "main".to_string()];

    let c = make_fingerprint("div", "item", "Content", Some("footer"), 0, 2);

    let sim_ab = AdaptiveTracker::similarity(&a, &b);
    let sim_ac = AdaptiveTracker::similarity(&a, &c);
    assert!(sim_ab > sim_ac);
}

#[test]
fn test_max_saved_limit() {
    let mut tracker = AdaptiveTracker::new_with_limit(3);
    for i in 0..5 {
        let fp = make_fingerprint("div", "", &format!("item {}", i), None, i, 1);
        tracker.save_element(&format!("elem_{}", i), "div", fp);
    }
    assert_eq!(tracker.saved.len(), 3);
    let names: Vec<&str> = tracker.saved.iter().map(|e| e.name.as_str()).collect();
    assert!(!names.contains(&"elem_0"));
    assert!(!names.contains(&"elem_1"));
}

#[test]
fn test_hit_count_tracking() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "content", "Hello", Some("main"), 0, 1);
    tracker.save_element("hit_test", "div.content", fp);

    let dom = DomSnapshot::new(vec![make_dom_element(
        "div",
        "content",
        "Hello",
        Some("main"),
        0,
        1,
    )]);

    assert_eq!(tracker.saved[0].hit_count, 0);

    tracker.locate("hit_test", &dom);
    assert_eq!(tracker.saved[0].hit_count, 1);

    tracker.locate("hit_test", &dom);
    assert_eq!(tracker.saved[0].hit_count, 2);
}

#[test]
fn test_dom_snapshot_creation() {
    let elements = vec![
        make_dom_element("div", "main", "Hello", None, 0, 1),
        make_dom_element("p", "text", "Paragraph", Some("div"), 0, 2),
    ];
    let snapshot = DomSnapshot::new(elements);
    assert_eq!(snapshot.elements.len(), 2);
    assert_eq!(snapshot.elements[0].tag, "div");
    assert_eq!(snapshot.elements[1].tag, "p");
}

#[test]
fn test_empty_tracker_locate() {
    let mut tracker = AdaptiveTracker::new();
    let dom = DomSnapshot::new(vec![make_dom_element(
        "div", "content", "Hello", None, 0, 1,
    )]);
    let result = tracker.locate("nonexistent", &dom);
    assert!(result.is_none());
}

#[test]
fn test_fuzzy_match_custom_threshold() {
    let tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "card", "Product Title", Some("section"), 1, 2);

    let dom = DomSnapshot::new(vec![
        make_dom_element("div", "card", "Product Title", Some("section"), 1, 2),
        make_dom_element("div", "box", "Something Else", Some("footer"), 3, 3),
    ]);

    let strict = tracker.find_similar(&fp, &dom, 0.95);
    assert_eq!(strict.len(), 1);
    assert!((strict[0].similarity - 1.0).abs() < 0.01);

    let loose = tracker.find_similar(&fp, &dom, 0.3);
    assert_eq!(loose.len(), 2);
}

#[test]
fn test_locate_nonexistent_name() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "a", "x", None, 0, 1);
    tracker.save_element("existing", "div.a", fp);
    let dom = DomSnapshot::new(vec![]);
    assert!(tracker.locate("nonexistent", &dom).is_none());
}

#[test]
fn test_similarity_href_and_src() {
    let mut a = make_fingerprint("a", "link", "Click", Some("nav"), 0, 1);
    a.href = Some("https://example.com".to_string());

    let mut b = make_fingerprint("a", "link", "Click", Some("nav"), 0, 1);
    b.href = Some("https://example.com".to_string());

    let mut c = make_fingerprint("a", "link", "Click", Some("nav"), 0, 1);
    c.href = None;

    let sim_ab = AdaptiveTracker::similarity(&a, &b);
    let sim_ac = AdaptiveTracker::similarity(&a, &c);
    assert!((sim_ab - sim_ac).abs() < 0.01);
}

#[test]
fn test_element_snapshot_creation() {
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("class".to_string(), "content main".to_string());
    attrs.insert("id".to_string(), "main-content".to_string());
    let snapshot = ElementSnapshot::new(
        "div",
        attrs,
        "Hello World",
        "/html/body/div[1]",
        vec!["div.content".to_string(), "#main-content".to_string()],
        Some("body"),
        vec!["container".to_string()],
        0,
        2,
    );
    assert_eq!(snapshot.tag, "div");
    assert_eq!(snapshot.primary_css_selector(), "div.content");
    assert_eq!(snapshot.xpath, "/html/body/div[1]");
    assert_eq!(snapshot.all_css_selectors().len(), 2);
}

#[test]
fn test_element_snapshot_to_fingerprint() {
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("class".to_string(), "card".to_string());
    attrs.insert("data-id".to_string(), "42".to_string());
    let snapshot = ElementSnapshot::new(
        "div",
        attrs,
        "Item",
        "/div[2]",
        vec!["div.card".to_string()],
        Some("section"),
        vec![],
        0,
        2,
    );
    let fp = snapshot.to_fingerprint();
    assert_eq!(fp.tag, "div");
    assert!(fp.data_attrs.contains_key("data-id"));
    assert!(fp.class_names.contains(&"card".to_string()));
}

#[test]
fn test_track_element_returns_fallbacks() {
    let mut tracker = AdaptiveTracker::new();
    let dom = DomSnapshot::new(vec![
        make_dom_element("div", "content", "Main", Some("body"), 0, 1),
        make_dom_element("section", "main", "Content", Some("body"), 1, 1),
    ]);
    let fallbacks = tracker.track_element("main", "div.content", &dom);
    assert_eq!(fallbacks.primary(), "div.content");
    assert!(tracker.saved.iter().any(|e| e.name == "main"));
}

#[test]
fn test_smart_selector_prefers_stable() {
    let mut tracker = AdaptiveTracker::new();
    let fp = make_fingerprint("div", "content", "Main", Some("body"), 0, 1);
    tracker.save_element("main", "div.content", fp);

    let dom = DomSnapshot::new(vec![make_dom_element(
        "section",
        "main-wrap",
        "Main",
        Some("body"),
        0,
        1,
    )]);
    let sel = AdaptiveTracker::smart_selector(&tracker.saved[0], &dom);
    assert!(!sel.is_empty());
}

#[test]
fn test_fallback_selectors_ordering() {
    let mut fs = FallbackSelectors::new("div.primary");
    fs.add("section.fallback", 0.7);
    fs.add("span.other", 0.3);
    fs.sort();
    assert_eq!(fs.primary(), "div.primary");
    assert_eq!(fs.fallbacks().len(), 2);
}

#[test]
fn test_fallback_selectors_dedup() {
    let mut fs = FallbackSelectors::new("div.same");
    fs.add("div.same", 0.5);
    assert_eq!(fs.all().len(), 1);
}
