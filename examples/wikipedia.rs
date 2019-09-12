fn main() {
    use wikidump::{config, Parser};

    let parser = Parser::new().use_config(config::wikipedia::english());
    let site = parser
        .parse_file("tests/enwiki-articles-partial.xml")
        .expect("Could not parse wikipedia dump file.");

    assert_eq!(site.name, "Wikipedia");
    assert_eq!(site.url, "https://en.wikipedia.org/wiki/Main_Page");
    assert!(!site.pages.is_empty());

    for page in site.pages {
        println!("\nTitle: {}", page.title);

        for revision in page.revisions {
            println!("\t{}", revision.text);
        }
    }
}
