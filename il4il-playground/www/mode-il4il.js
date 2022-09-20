import { parser } from "./il4il.grammar"
import * as language from "@codemirror/language"
import * as highlight from "@lezer/highlight"
import * as autocomplete from "@codemirror/autocomplete"

const LANGUAGE = language.LRLanguage.define({
    parser: parser.configure({
        props: [
            language.indentNodeProp.add({
                Block: language.delimitedIndent({ closing: "}", align: false }),
            }),
            highlight.styleTags({
                Word: highlight.tags.keyword,
                Directive: highlight.tags.definitionKeyword,
                String: highlight.tags.string,
                Integer: highlight.tags.integer,
                "{ }": highlight.tags.bracket,
            }),
        ]
    }),
});

const COMPLETION = LANGUAGE.data.of({
    autocomplete: autocomplete.completeFromList([
        { label: ".section", type: "keyword" }
    ])
});

export function il4il() {
    return new language.LanguageSupport(LANGUAGE, [COMPLETION]);
}
