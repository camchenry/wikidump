use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::fs::File;
use std::time::Duration;
use wikidump::{config, Parser, Site};

fn parse_wikipedia(file: &'static str, parse_wiki_text: bool) -> Site {
    let parser = Parser::new()
        .process_text(parse_wiki_text)
        .remove_newlines(true)
        .use_config(config::wikipedia::english());
    parser.parse_file(file).expect("Failed to parse")
}

fn wikipedia_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Wikipedia");

    let file_length = File::open("benches/enwiki-10k.xml")
        .unwrap()
        .metadata()
        .unwrap()
        .len();
    group.sample_size(60);
    group.measurement_time(Duration::new(15, 0));
    group.throughput(Throughput::Bytes(file_length));
    group.bench_function("enwiki_10k_with_parsing_wiki_text", |b| {
        b.iter(|| parse_wikipedia(black_box("benches/enwiki-10k.xml"), black_box(true)))
    });
    group.bench_function("enwiki_10k_no_parsing_wiki_text", |b| {
        b.iter(|| parse_wikipedia(black_box("benches/enwiki-10k.xml"), black_box(false)))
    });

    group.finish();
}

fn simplewiki_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Simple Wikipedia");

    let file_length = File::open("tests/simplewiki.xml")
        .unwrap()
        .metadata()
        .unwrap()
        .len();
    group.measurement_time(Duration::new(10, 0));
    group.throughput(Throughput::Bytes(file_length));
    group.bench_function("simplewiki_partial_with_parsing_wiki_text", |b| {
        b.iter(|| parse_wikipedia(black_box("tests/simplewiki.xml"), black_box(true)))
    });
    group.bench_function("simplewiki_partial_no_parsing_wiki_text", |b| {
        b.iter(|| parse_wikipedia(black_box("tests/simplewiki.xml"), black_box(false)))
    });

    group.finish();
}

criterion_group!(benches, wikipedia_benchmark, simplewiki_benchmark);
criterion_main!(benches);
