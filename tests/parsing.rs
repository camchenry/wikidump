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
            .use_config(config::wikipedia::simple_english())
            .exclude_pages(false)
            .remove_newlines(true);

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

        assert_eq!(
            revision.text.split(' ').take(7).collect::<Vec<&str>>(),
            vec!("Art", "and", "crafts", "is", "a", "creative", "activity")
        );
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

    #[test]
    fn does_remove_newlines() {
        let parser = Parser::new().remove_newlines(true);

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        for page in site.pages {
            for revision in page.revisions {
                assert!(!revision.text.contains("\n"));
            }
        }
    }

    // Wikipedia tests
    #[test]
    fn can_parse_enwiki_siteinfo() {
        let parser = Parser::new().use_config(config::wikipedia::english());

        let site = parser
            .parse_file("tests/enwiki-articles-partial.xml")
            .expect("Could not parse enwiki dump");

        assert_eq!(site.name, "Wikipedia");
        assert_eq!(site.url, "https://en.wikipedia.org/wiki/Main_Page");
    }

    #[test]
    fn can_parse_enwiki_pages() {
        let parser = Parser::new()
            .use_config(config::wikipedia::english())
            .exclude_pages(false);

        let site = parser
            .parse_file("tests/enwiki-articles-partial.xml")
            .expect("Could not parse simplewiki dump");

        assert!(!site.pages.is_empty(), "Site page list is empty");
        assert_eq!(site.pages.len(), 11);

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == "Ricky Minard".to_string())
            .expect("Could not fetch example page");
        assert_eq!(page.title, "Ricky Minard");

        assert!(!page.revisions.is_empty(), "Found no revisions for page");
        assert_eq!(page.revisions.len(), 1);

        let revision = page
            .revisions
            .first()
            .expect("Could not get first revision");

        assert_eq!(
            revision.text.split(" ").take(7).collect::<Vec<&str>>(),
            vec!(
                "Ricky",
                "Donell",
                "Minard",
                "Jr.",
                "(born",
                "September",
                "11,"
            )
        );
    }

    const MEDIAWIKI_DUMP: &str = r#"
        <mediawiki xmlns="http://www.mediawiki.org/xml/export-0.10/">
            <page>
                <ns>0</ns>
                <title>alpha</title>
                <revision>
                    <text></text>
                </revision>
            </page>
            <page>
                <ns>42</ns>
                <title>beta</title>
                <revision>
                    <text></text>
                </revision>
            </page>
        </mediawiki>
    "#;

    #[test]
    fn can_parse_str() {
        let parser = Parser::new();

        let site = parser
            .parse_str(MEDIAWIKI_DUMP)
            .expect("Could not parse mediawiki dump");

        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].title, "alpha");
    }

    #[test]
    fn will_exclude_pages() {
        let parser = Parser::new();

        let site = parser
            .parse_str(MEDIAWIKI_DUMP)
            .expect("Could not parse mediawiki dump");

        assert_eq!(site.pages.len(), 1);
        assert_eq!(site.pages[0].title, "alpha");
    }

    #[test]
    fn will_not_exclude_pages() {
        let parser = Parser::new().exclude_pages(false);

        let site = parser
            .parse_str(MEDIAWIKI_DUMP)
            .expect("Could not parse mediawiki dump");

        assert_eq!(site.pages.len(), 2);
    }
}
