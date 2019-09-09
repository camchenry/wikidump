use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

type Exception = Box<dyn std::error::Error + 'static>;

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
}

#[derive(Debug)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub pages: Vec<Page>,
}

#[derive(Debug)]
pub struct DumpParser {}

#[derive(Debug, PartialEq)]
enum ParserState {
    None,
    SiteInfo,
    Pages,
}

impl DumpParser {
    pub fn new() -> DumpParser {
        DumpParser {}
    }

    pub fn parse_file<P>(&self, dump: P) -> Result<Site, Exception>
    where
        P: AsRef<Path>,
    {
        let mut reader = Reader::from_file(dump).expect("Could not create XML reader from file");
        reader.trim_text(true);

        // TODO
        let mut site = Site {
            name: "".to_string(),
            url: "".to_string(),
            pages: vec![],
        };

        let mut state = ParserState::None;
        let mut buf = Vec::new();
        let mut text_buf = Vec::new();
        let mut current_page = Page {
            title: "".to_string(),
        };

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element_name = e.name();
                    // Check if we should change between states
                    match element_name {
                        b"siteinfo" => state = ParserState::SiteInfo,
                        b"page" => state = ParserState::Pages,
                        _ => {}
                    };

                    if state == ParserState::SiteInfo {
                        // Site information: URL, name, etc.

                        match element_name {
                            b"sitename" => {
                                site.name = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get site name");
                            }
                            b"base" => {
                                site.url = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get base wiki URL");
                            }
                            _ => {}
                        };
                    } else if state == ParserState::Pages {
                        // Page information: title, text, authors, etc.
                        match element_name {
                            b"title" => {
                                current_page.title = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get page title");
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Text(e)) => {}
                Ok(Event::End(ref e)) => {
                    if state == ParserState::Pages {
                        site.pages.push(current_page.clone());
                    }
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
            text_buf.clear();
        }

        Ok(site)
    }
}
