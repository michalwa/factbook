import {
  LRLanguage,
  HighlightStyle,
  syntaxHighlighting,
} from "@codemirror/language";
import { parser } from "./viewParser.gen";
import { styleTags, tags as t } from "@lezer/highlight";

const viewHighlight = styleTags({
  Comment: t.comment,
  Operator: t.operator,
  "Ident Quoted": t.atom,
  "Args/Ident Args/Quoted": t.function(t.atom),
  Variable: t.variableName,
  Number: t.number,
  String: t.string,
  "@": t.punctuation,
  ", ;": t.separator,
  "( )": t.paren,
  "[ ]": t.squareBracket,
  "{ }": t.brace,
});

const viewHighlightStyle = HighlightStyle.define([
  { tag: t.comment, class: "cm-highlight-comment" },
  { tag: t.punctuation, class: "cm-highlight-punctuation" },
  { tag: t.atom, class: "cm-highlight-ident" },
  {
    tag: t.function(t.atom),
    class: "cm-highlight-argument cm-highlight-ident",
  },
  { tag: t.variableName, class: "cm-highlight-variable" },
  { tag: t.number, class: "cm-highlight-number" },
  { tag: t.string, class: "cm-highlight-string" },
]);

const viewLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [viewHighlight],
  }),
});

export const viewLanguageExtension = [
  viewLanguage,
  syntaxHighlighting(viewHighlightStyle),
];
