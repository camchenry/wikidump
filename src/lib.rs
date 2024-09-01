//! This crate can process Mediawiki dump (backup) files in XML format and
//! allow you to extract whatever data you desire.
//!
//! # Example
//! ```rust
//! use wikidump::{config, Parser};
//!
//! let parser = Parser::new().use_config(config::wikipedia::english());
//! let site = parser
//!     .parse_file("tests/enwiki-articles-partial.xml")
//!     .expect("Could not parse wikipedia dump file.");
//!
//! assert_eq!(site.name, "Wikipedia");
//! assert_eq!(site.url, "https://en.wikipedia.org/wiki/Main_Page");
//! assert!(!site.pages.is_empty());
//!
//! for page in site.pages {
//!     println!("\nTitle: {}", page.title);
//!
//!     for revision in page.revisions {
//!         println!("\t{}", revision.text);
//!     }
//! }
//! ```

pub mod config;
use bzip2::read::MultiBzDecoder;
use parse_wiki_text::{Configuration, ConfigurationSource, Node};
use quick_xml::events::Event;
use quick_xml::Reader;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

type Exception = Box<dyn std::error::Error + 'static>;

/// Represents a wiki page.
#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub revisions: Vec<PageRevision>,
}

impl Page {
    /// Creates a new page with no data.
    fn new() -> Page {
        Page {
            title: "".to_string(),
            revisions: vec![],
        }
    }

    /// Reset internal data without allocating.
    fn reset(&mut self) -> &Self {
        self.title.clear();
        self.revisions.clear();
        self
    }
}

/// Represents a specific revision of a page. This means a certain version of
/// the page a specific time with some text contents which was created by
/// some contributor.
#[derive(Debug, Clone)]
pub struct PageRevision {
    /// The text content of the page. Depending on whether the parser is
    /// processing wiki text or not, this could either be the raw wiki text
    /// or it could be an interpreted representation.
    pub text: String,
    pub raw: String,
}

impl PageRevision {
    fn new() -> PageRevision {
        PageRevision {
            text: "".to_string(),
            raw: "".to_string(),
        }
    }

    /// Reset internal data without allocating.
    fn reset(&mut self) -> &mut Self {
        self.text.clear();
        self
    }
}

/// Represents a Mediawiki website, like Wikipedia, for example.
#[derive(Debug)]
pub struct Site {
    /// The name of the website, e.g., "Wikipedia".
    pub name: String,
    /// The base URL of the website, e.g., "https://en.wikipedia.org/wiki/Main_Page".
    pub url: String,
    /// The wiki pages belonging to the website.
    pub pages: Vec<Page>,
}

impl Site {
    fn new() -> Site {
        Site {
            name: "".to_string(),
            url: "".to_string(),
            pages: vec![],
        }
    }
}

/// A parser which can process uncompressed Mediawiki XML dumps (backups).
pub struct Parser {
    /// If true, the wiki text will be parsed and turned into simple text which
    /// could be read naturally.
    process_wiki_text: bool,
    /// If true and processing wiki text is enabled, then newlines will be
    /// removed from the output. Otherwise, they are turned into actual newline
    /// characters.
    remove_newlines: bool,
    /// If true, then only pages which are articles (and not Talk or Special
    /// pages, or any other kind of page) will be included in the final output.
    /// Any ignored pages will simply be skipped by the parser.
    exclude_pages: bool,
    /// The specific wiki configuration for parsing.
    wiki_config: Configuration,
}

impl Parser {
    /// Construct a new parser with the default settings.
    pub fn new<'c>() -> Parser {
        Parser {
            process_wiki_text: true,
            remove_newlines: false,
            exclude_pages: true,
            wiki_config: Configuration::default(),
        }
    }

    /// Sets whether the parser should process wiki text or leave it as-is. For
    /// best results, it is recommended you use a wiki config which matches the
    /// website you are parsing from. It may still work otherwise, but the
    /// results might be something unexpected.
    ///
    /// Wiki text parsing is enabled by default.
    ///
    /// See [use_config](struct.Parser.html#method.use_config) and [config](config/index.html).
    ///
    /// # Example
    /// ```rust
    /// use wikidump::{Parser, config};
    ///
    /// let parser = Parser::new()
    ///     .use_config(config::wikipedia::english())
    ///     .process_text(false); // Disable wiki text parsing
    /// ```
    pub fn process_text(mut self, value: bool) -> Self {
        self.process_wiki_text = value;
        self
    }

    /// Sets whether the parser should ignore pages in namespaces that are not
    /// articles, such as Talk, Special, or User. If enabled, then any page
    /// which is not an article will be skipped by the parser.
    ///
    /// Excluding pages in these namespaces is enabled by default.
    ///
    /// # Example
    /// ```rust
    /// use wikidump::{Parser, config};
    ///
    /// let parser = Parser::new()
    ///     .use_config(config::wikipedia::english())
    ///     .exclude_pages(false); // Disable page exclusion
    /// ```
    pub fn exclude_pages(mut self, value: bool) -> Self {
        self.exclude_pages = value;
        self
    }

    /// Sets whether the parser should remove newlines or turn them into normal
    /// newline characters. This will only have an effect if processing wiki
    /// text is enabled.
    ///
    /// Removing newlines is turned off by default.
    ///
    /// # Example
    /// ```rust
    /// use wikidump::{Parser, config};
    ///
    /// let parser = Parser::new()
    ///     .use_config(config::wikipedia::english())
    ///     .remove_newlines(true) // Enable newline removal
    ///     .process_text(true);
    /// ```
    pub fn remove_newlines(mut self, value: bool) -> Self {
        self.remove_newlines = value;
        self
    }

    /// Sets the wiki text parser configuration options. For best results of
    /// processing wiki text, it is recommended to use the type of configuration
    /// that matches the website and language you are processing.
    ///
    /// See [config](config/index.html).
    ///
    /// # Example
    /// ```rust
    /// use wikidump::{Parser, config};
    ///
    /// let parser = Parser::new()
    ///     .use_config(config::wikipedia::english());
    /// ```
    pub fn use_config(mut self, config_source: ConfigurationSource) -> Self {
        self.wiki_config = Configuration::new(&config_source);
        self
    }

    /// Returns all of the parsed data contained in a particular wiki dump file.
    /// This includes the name of the website, a list of pages, their
    /// respective contents, and other properties.
    ///
    /// # Example
    /// ```rust
    /// use wikidump::Parser;
    ///
    /// let parser = Parser::new();
    /// let site = parser.parse_file("tests/enwiki-articles-partial.xml");
    /// ```
    pub fn parse_file<P>(&self, dump: P) -> Result<Site, Exception>
    where
        P: AsRef<Path>,
    {
        if is_compressed(&dump) {
            let file = File::open(dump)?;
            let reader = BufReader::new(MultiBzDecoder::new(file));
            let reader = Reader::from_reader(reader);

            self.parse(reader)
        } else {
            let reader = Reader::from_file(dump).expect("Could not create XML reader from file");

            self.parse(reader)
        }
    }

    /// Returns all of the parsed data contained in a particular wiki dump file.
    /// This includes the name of the website, a list of pages, their
    /// respective contents, and other properties.
    ///
    /// # Example
    /// ```rust
    /// use wikidump::Parser;
    /// use std::fs;
    ///
    /// let parser = Parser::new();
    /// let contents = fs::read_to_string("tests/enwiki-articles-partial.xml").unwrap();
    /// let site = parser.parse_str(contents.as_str());
    /// ```
    pub fn parse_str(&self, text: &str) -> Result<Site, Exception> {
        let reader = Reader::from_str(text);

        self.parse(reader)
    }

    fn parse<R>(&self, mut reader: Reader<R>) -> Result<Site, Exception>
    where
        R: BufRead,
    {
        // Save time by assuming well formed XML is passed in.
        reader.check_end_names(false);
        reader.trim_markup_names_in_closing_tags(false);

        let mut site = Site::new();
        let mut buf = Vec::new();
        let mut text_buf = Vec::new();
        let mut current_page = Page::new();
        let mut current_page_revision = PageRevision::new();
        let mut skipping_current_page = false;

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if skipping_current_page {
                        continue;
                    }
                    let element_name = e.name();

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
                        b"text" => {
                            current_page_revision.text = reader
                                .read_text(element_name, &mut text_buf)
                                .expect("Could not get revision text");
                        }
                        b"title" => {
                            current_page.title = reader
                                .read_text(element_name, &mut text_buf)
                                .expect("Could not get page title");
                        }
                        b"ns" => {
                            if self.exclude_pages {
                                let ns = reader
                                    .read_text(element_name, &mut text_buf)
                                    .expect("Could not get page namespace");

                                if ns != "0" {
                                    // Skip this page
                                    skipping_current_page = true;
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    };
                }
                Ok(Event::End(ref e)) => {
                    match e.name() {
                        b"page" => {
                            if !skipping_current_page {
                                site.pages.push(current_page.clone());
                                current_page.reset();
                            }

                            skipping_current_page = false;
                        }
                        b"revision" => {
                            current_page.revisions.push(current_page_revision.clone());
                            current_page_revision.reset();
                        }
                        _ => {}
                    };
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                _ => (), // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
            text_buf.clear();
        }

        site.pages.par_iter_mut().for_each(|p: &mut Page| {
            p.revisions.par_iter_mut().for_each(|r: &mut PageRevision| {
                if self.process_wiki_text {
                    let parsed_output = self.wiki_config.parse(r.text.as_str());

                    r.raw = r.text.as_str().to_string();
                    r.text = get_text_from_nodes(&parsed_output.nodes).replace("\\t", "");
                }

                if self.remove_newlines {
                    r.text = r.text.replace("\n", "");
                    r.text = r.text.replace("\r", "");
                }

                r.text = r.text.trim().to_string();
            })
        });

        Ok(site)
    }
}

// TODO: document
fn get_text_from_nodes(nodes: &Vec<Node>) -> String {
    // 32 is just a guess here, not really well benchmarked or anything
    let mut node_text = String::with_capacity(64 + 64 * nodes.len());

    nodes.iter().for_each(|node| {
        match node {
            Node::Text { value, .. } => node_text.push_str(value),
            Node::ParagraphBreak { .. } => node_text.push_str("\n"),
            Node::CharacterEntity { character, .. } => {
                node_text.push_str(character.to_string().as_str())
            }
            Node::Link { text, .. } => node_text.push_str(get_text_from_nodes(text).as_str()),
            Node::ExternalLink { nodes, .. } => {
                node_text.push_str(get_text_from_nodes(nodes).as_str())
            }
            Node::Heading { nodes, .. } => {
                node_text.push_str("\n");
                node_text.push_str(get_text_from_nodes(nodes).as_str());
                node_text.push_str("\n");
            }
            Node::Image { .. } => {
                // @TODO @Completeness: Allow image text.
                // Currently not allowed because it's a bit difficult to figure
                // out what is normal text and what isn't.
            }
            Node::OrderedList { items, .. } | Node::UnorderedList { items, .. } => {
                items.iter().for_each(|i| {
                    node_text.push_str(get_text_from_nodes(&i.nodes).as_str());
                });
            }
            Node::DefinitionList { items, .. } => {
                items.iter().for_each(|i| {
                    node_text.push_str(get_text_from_nodes(&i.nodes).as_str());
                });
            }
            Node::Preformatted { nodes, .. } => {
                node_text.push_str(get_text_from_nodes(nodes).as_str())
            }
            Node::Template { .. }
            | Node::Bold { .. }
            | Node::BoldItalic { .. }
            | Node::HorizontalDivider { .. }
            | Node::MagicWord { .. }
            | Node::Italic { .. }
            | Node::Redirect { .. }
            | Node::Comment { .. }
            | Node::Tag { .. }
            | Node::StartTag { .. }
            | Node::EndTag { .. }
            | Node::Parameter { .. }
            | Node::Category { .. }
            | Node::Table { .. } => {}
        }
    });

    node_text
}

fn is_compressed<P>(dump: &P) -> bool
where
    P: AsRef<Path>,
{
    let bytes_to_read = 3;
    let mut buf = vec![0u8; bytes_to_read];
    let mut file = File::open(dump).expect("Could not open dump file");
    file.read_exact(&mut buf).expect("Could not read dump file");
    buf == b"BZh"
}
