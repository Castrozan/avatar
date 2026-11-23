use avatar_audio_graph::pulse::listing::parse_modules;
use avatar_audio_graph::pulse::module_record::ModuleRecord;
use proptest::prelude::*;

fn render(module: &ModuleRecord) -> String {
    format!("{}\t{}\t{}", module.index, module.name, module.argument)
}

proptest! {
    #[test]
    fn rendered_modules_parse_back_unchanged(
        modules in proptest::collection::vec(
            (any::<u32>(), "[a-z][a-z-]{0,20}", "[A-Za-z0-9_=. ]{0,40}")
                .prop_map(|(index, name, argument)| ModuleRecord::new(index, &name, argument)),
            0..8,
        ),
    ) {
        let listing: Vec<String> = modules.iter().map(render).collect();
        prop_assert_eq!(parse_modules(&listing.join("\n")), modules);
    }

    #[test]
    fn arbitrary_text_never_yields_more_modules_than_lines(text in ".{0,300}") {
        prop_assert!(parse_modules(&text).len() <= text.lines().count());
    }
}
