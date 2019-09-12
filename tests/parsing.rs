#[cfg(test)]
mod tests {
    use wikidump::config;
    use wikidump::Parser;

    #[test]
    fn can_create_parser() {
        let _parser = Parser::new();
    }

    #[test]
    fn can_set_parser_options() {
        let _parser = Parser::new()
            .process_text(true)
            .use_config(config::wikipedia::simple_english());
    }

    #[test]
    fn can_parse_simplewiki_siteinfo() {
        let parser = Parser::new().use_config(config::wikipedia::simple_english());

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        assert_eq!(site.name, "Wikipedia");
        assert_eq!(site.url, "https://simple.wikipedia.org/wiki/Main_Page");
    }

    #[test]
    fn can_parse_simplewiki_pages() {
        let parser = Parser::new()
            .process_text(true)
            .use_config(config::wikipedia::simple_english());

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        assert!(!site.pages.is_empty(), "Site page list is empty");
        assert_eq!(site.pages.len(), 7);

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == "Art".to_string())
            .expect("Could not fetch example page");
        assert_eq!(page.title, "Art");

        assert!(!page.revisions.is_empty(), "Found no revisions for page");
        assert_eq!(page.revisions.len(), 1);

        let revision = page
            .revisions
            .first()
            .expect("Could not get first revision");

        assert!(revision
            .text
            .split(" ")
            .collect::<Vec<&str>>()
            .starts_with(&vec!(
                "Art", "and", "crafts", "is", "a", "creative", "activity"
            )));
    }

    #[test]
    fn can_disable_text_parsing() {
        let parser = Parser::new().process_text(false);

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == "Art".to_string())
            .expect("Could not fetch example page");

        let revision = page
            .revisions
            .first()
            .expect("Could not get first revision");

        assert!(revision.text.contains("[["));
        assert!(revision.text.contains("]]"));
    }
}
