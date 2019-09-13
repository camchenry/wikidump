[![Build Status](__badge_image__)](__badge_url__)

# wikidump

This crate can process Mediawiki dump (backup) files in XML format and
allow you to extract whatever data you desire.

## Example
```rust
let parser = Parser::new()
    .use_config(config::wikipedia::english());

let site = parser
    .parse_file("tests/enwiki-articles-partial.xml")
    .expect("Could not parse wikipedia dump file.");

assert_eq!(site.name, "Wikipedia");
assert_eq!(site.url, "https://en.wikipedia.org/wiki/Main_Page");
assert!(!site.pages.is_empty());

for page in site.pages {
    println!("Title: {}", page.title);

    for revision in page.revisions {
        println!("\t{}", revision.text);
    }
}
```
