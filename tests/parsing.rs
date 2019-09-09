#[cfg(test)]
mod tests {
    use std::fs::File;
    use wikidump::{DumpParser, Site};

    #[test]
    fn can_create_parser() {
        let _parser = DumpParser::new();
    }

    #[test]
    fn can_parse_simplewiki_dump() {
        let parser = DumpParser::new();

        let site = parser
            .parse_file("tests/simplewiki.xml")
            .expect("Could not parse simplewiki dump");

        assert_eq!(site.name, "Wikipedia");
        assert_eq!(site.url, "https://simple.wikipedia.org/wiki/Main_Page");
    }
}
