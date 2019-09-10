#[cfg(test)]
mod tests {
    use wikidump::DumpParser;

    #[test]
    fn can_create_parser() {
        let _parser = DumpParser::new();
    }

    #[test]
    fn can_parse_simplewiki_siteinfo() {
        let parser = DumpParser::new();

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        assert_eq!(site.name, "Wikipedia");
        assert_eq!(site.url, "https://simple.wikipedia.org/wiki/Main_Page");
    }

    #[test]
    fn can_parse_simplewiki_pages() {
        let parser = DumpParser::new();

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        assert!(!site.pages.is_empty(), "Site page list is empty");

        let page = site
            .pages
            .iter()
            .find(|&p| p.title == "Art".to_string())
            .expect("Could not fetch example page");
        assert_eq!(page.title, "Art");

        assert!(!page.revisions.is_empty(), "Found no revisions for page");

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
}
