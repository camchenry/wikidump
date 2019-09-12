pub mod config;
use parse_wiki_text::{Configuration, ConfigurationSource, Node};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

type Exception = Box<dyn std::error::Error + 'static>;

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub revisions: Vec<PageRevision>,
}

impl Page {
    pub fn new() -> Page {
        Page {
            title: "".to_string(),
            revisions: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageRevision {
    pub text: String,
}

impl PageRevision {
    pub fn new() -> PageRevision {
        PageRevision {
            text: "".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Site {
    pub name: String,
    pub url: String,
    pub pages: Vec<Page>,
}

impl Site {
    pub fn new() -> Site {
        Site {
            name: "".to_string(),
            url: "".to_string(),
            pages: vec![],
        }
    }
}

pub struct DumpParser {
    process_wiki_text: bool,
    wiki_config: Configuration,
}

#[derive(Debug, PartialEq)]
enum ParserState {
    None,
    SiteInfo,
    Page,
}

impl DumpParser {
    pub fn new<'c>() -> DumpParser {
        DumpParser {
            process_wiki_text: true,
            wiki_config: Configuration::default(),
        }
    }

    pub fn process_text(mut self, value: bool) -> Self {
        self.process_wiki_text = value;
        self
    }

    pub fn use_config(mut self, config_source: ConfigurationSource) -> Self {
        self.wiki_config = Configuration::new(&config_source);
        self
    }

    pub fn parse_file<P>(&self, dump: P) -> Result<Site, Exception>
    where
        P: AsRef<Path>,
    {
        let mut reader = Reader::from_file(dump).expect("Could not create XML reader from file");
        reader.trim_text(true);

        // TODO
        let mut site = Site::new();
        let mut state = ParserState::None;
        let mut buf = Vec::new();
        let mut text_buf = Vec::new();
        let mut hierarchy: Vec<String> = Vec::new();
        let mut current_page = Page::new();
        let mut current_page_revision = PageRevision::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let element_name = e.name();
                    let new_name = std::str::from_utf8(element_name).unwrap().to_string();
                    hierarchy.push(new_name);

                    // Check if we should change between states
                    match element_name {
                        b"siteinfo" => state = ParserState::SiteInfo,
                        b"page" => state = ParserState::Page,
                        _ => {}
                    };

                    if state == ParserState::SiteInfo {
                        // Site information: URL, name, etc.
                        match element_name {
                            b"sitename" => {
                                site.name = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get site name");
                                hierarchy.pop(); // Pop hierarchy because we processed this node in-place.
                            }
                            b"base" => {
                                site.url = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get base wiki URL");
                                hierarchy.pop(); // Pop hierarchy because we processed this node in-place.
                            }
                            _ => {}
                        };
                    } else if state == ParserState::Page {
                        // Page information: title, text, authors, etc.
                        // IMPORTANT: These tag names are ordered from deepest to shallowest.
                        if hierarchy.contains(&"revision".to_string()) {
                            match element_name {
                                b"text" => {
                                    // @TODO @Completeness: Provide an option here to NOT
                                    // parse the wiki text, just in case.
                                    let text = reader
                                        .read_text(element_name, &mut text_buf)
                                        .expect("Could not get revision text");

                                    if self.process_wiki_text {
                                        // @TODO: Allow swapping the configuration
                                        let parsed_result = self.wiki_config.parse(text.as_str());

                                        let text = get_text_from_nodes(parsed_result.nodes);

                                        current_page_revision.text = text;
                                    } else {
                                        current_page_revision.text = text;
                                    }

                                    hierarchy.pop(); // Pop hierarchy because we processed this node in-place.
                                }
                                _ => {}
                            };
                        } else if hierarchy.contains(&"page".to_string()) {
                            match element_name {
                                b"title" => {
                                    current_page.title = reader
                                        .read_text(element_name, &mut text_buf)
                                        .expect("Could not get page title");
                                    hierarchy.pop(); // Pop hierarchy because we processed this node in-place.
                                }
                                _ => {}
                            };
                        }
                    }
                }
                Ok(Event::Text(e)) => {}
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        b"page" => {
                            site.pages.push(current_page.clone());
                            // TODO(performance): Do not allocate here, instead do a reset.
                            current_page = Page::new();
                        }
                        b"revision" => {
                            current_page.revisions.push(current_page_revision.clone());
                            // TODO(performance): Do not allocate here, instead do a reset.
                            current_page_revision = PageRevision::new();
                        }
                        _ => {}
                    };

                    hierarchy.pop();
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

fn get_text_from_nodes(nodes: Vec<Node>) -> String {
    let mut node_text = "".to_string();

    for node in nodes {
        match node {
            Node::Text { value, .. } => node_text.push_str(value),
            Node::CharacterEntity { character, .. } => {
                node_text.push_str(character.to_string().as_str())
            }
            Node::Link { text, .. } => node_text.push_str(get_text_from_nodes(text).as_str()),
            Node::ExternalLink { nodes, .. } => {
                node_text.push_str(get_text_from_nodes(nodes).as_str())
            }
            Node::Heading { nodes, .. } => node_text.push_str(get_text_from_nodes(nodes).as_str()),
            Node::Image { .. } => {
                // @TODO @Completeness: Allow image text.
                // Currently not allowed because it's a bit difficult to figure
                // out what is normal text and what isn't.
            }
            Node::UnorderedList { items, .. } => {
                for item in items {
                    node_text.push_str(get_text_from_nodes(item.nodes).as_str());
                }
            }
            Node::DefinitionList { items, .. } => {
                for item in items {
                    node_text.push_str(get_text_from_nodes(item.nodes).as_str());
                }
            }
            Node::Template { .. }
            | Node::Bold { .. }
            | Node::ParagraphBreak { .. }
            | Node::Italic { .. }
            | Node::Tag { .. }
            | Node::StartTag { .. }
            | Node::EndTag { .. }
            | Node::Category { .. }
            | Node::Table { .. } => {}
            _ => panic!("Unhandled node type: {:?}", node),
        }
    }

    node_text
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .trim()
        .to_string()
}
