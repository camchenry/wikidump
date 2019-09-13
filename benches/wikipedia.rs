use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::fs::File;
use std::time::Duration;
use wikidump::{config, Parser, Site};

fn parse_wikipedia(file: &'static str, parse_wiki_text: bool) -> Site {
    let parser = Parser::new()
        .process_text(parse_wiki_text)
        .use_config(config::wikipedia::english());
    parser.parse_file(file).expect("Failed to parse")
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Wikipedia");

    let file_length = File::open("tests/enwiki-articles-partial.xml")
        .unwrap()
        .metadata()
        .unwrap()
        .len();
    group.measurement_time(Duration::new(10, 0));
    group.throughput(Throughput::Bytes(file_length));
    group.bench_function("enwiki partial w/ parsing", |b| {
        b.iter(|| {
            parse_wikipedia(
                black_box("tests/enwiki-articles-partial.xml"),
                black_box(true),
            )
        })
    });
    group.bench_function("enwiki partial w/ no parsing", |b| {
        b.iter(|| {
            parse_wikipedia(
                black_box("tests/enwiki-articles-partial.xml"),
                black_box(false),
            )
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
