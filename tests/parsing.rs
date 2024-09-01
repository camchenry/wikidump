#[cfg(test)]
mod tests {
    use wikidump::config;
    use wikidump::Parser;

    #[test]
    fn can_create_parser() {
        let _parser = Parser::new();
    }

    #[test]
    fn can_be_constructed_with_default() {
        let _parser = Parser::default();
    }

    #[test]
    fn can_set_parser_options() {
        let _parser = Parser::new()
            .process_text(true)
            .use_config(config::wikipedia::simple_english());
    }

    // Simplewiki tests
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
            .find(|&p| p.title == *"Art")
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
            .find(|&p| p.title == *"Art")
            .expect("Could not fetch example page");

        let revision = page
            .revisions
            .first()
            .expect("Could not get first revision");

        assert!(revision.text.contains("[["));
        assert!(revision.text.contains("]]"));
    }

    #[test]
    fn can_access_raw_text() {
        let parser = Parser::new().process_text(true);
        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == *"Art")
            .expect("Could not fetch example page");

        let revision = page
            .revisions
            .first()
            .expect("Could not get first revision");

        assert_eq!(&revision.text[..15], "Art and crafts ");
        assert_eq!(&revision.raw[..15], "[[Category:Art|");
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
            .find(|&p| p.title == *"Ricky Minard")
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

    const NEWLINE_TEST: &str = r#"
        <mediawiki xmlns="http://www.mediawiki.org/xml/export-0.10/">
            <page>
                <ns>0</ns>
                <title>alpha</title>
                <revision>
                    <text>this is a test     \t\n\r
                        this is a test \t\r\n</text>
                </revision>
            </page>
        </mediawiki>
    "#;

    #[test]
    fn does_remove_newlines() {
        let parser = Parser::new().remove_newlines(true);

        let site = parser
            .parse_str(NEWLINE_TEST)
            .expect("Could not parse newline test str");

        for page in site.pages {
            for revision in page.revisions {
                assert!(!revision.text.contains('\n'));
                assert!(!revision.text.contains('\r'));
            }
        }

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        for page in site.pages {
            for revision in page.revisions {
                assert!(!revision.text.contains('\n'));
                assert!(!revision.text.contains('\r'));
            }
        }
    }

    // Simplewiki (compressed) tests
    #[test]
    fn can_parse_bz2_simplewiki_siteinfo() {
        let parser = Parser::new().use_config(config::wikipedia::simple_english());

        let site = parser
            .parse_file("tests/simplewiki.xml.bz2")
            .expect("Could not parse simplewiki dump");

        assert_eq!(site.name, "Wikipedia");
        assert_eq!(site.url, "https://simple.wikipedia.org/wiki/Main_Page");
    }

    #[test]
    fn can_parse_bz2_simplewiki_pages() {
        let parser = Parser::new()
            .use_config(config::wikipedia::simple_english())
            .exclude_pages(false)
            .remove_newlines(true);

        let site = parser
            .parse_file("tests/simplewiki.xml.bz2")
            .expect("Could not parse simplewiki dump");

        assert!(!site.pages.is_empty(), "Site page list is empty");
        assert_eq!(site.pages.len(), 7);

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == *"Art")
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

    const TEXT_TEST: &str = r#"
        <mediawiki xmlns="http://www.mediawiki.org/xml/export-0.10/">
            <page>
                <ns>0</ns>
                <title>alpha</title>
                <revision>
                    <text>This is an article.
== Header ==
This is text under the header.</text>
                </revision>
            </page>
            <page>
                <ns>0</ns>
                <title>beta</title>
                <revision>
                    <text>This is paragraph 1.

This is paragraph 2.</text>
                </revision>
            </page>
        </mediawiki>
    "#;

    #[test]
    fn turns_headers_into_text_with_newlines() {
        let parser = Parser::new();
        let site = parser
            .parse_str(TEXT_TEST)
            .expect("Could not parse text test str");

        let text = &site.pages[0].revisions[0].text;

        assert_eq!(
            text,
            "This is an article.\nHeader\nThis is text under the header."
        );
    }

    #[test]
    fn turns_paragraph_breaks_into_newlines() {
        let parser = Parser::new();
        let site = parser
            .parse_str(TEXT_TEST)
            .expect("Could not parse text test str");

        let text = &site.pages[1].revisions[0].text;

        assert_eq!(text, "This is paragraph 1.\nThis is paragraph 2.");
    }
}
