import {
  LRLanguage,
  HighlightStyle,
  syntaxHighlighting,
} from "@codemirror/language";
import { parser } from "./viewParser.gen";
import { styleTags, tags as t } from "@lezer/highlight";
import { autocompletion } from "@codemirror/autocomplete";
import { syntaxTree } from "@codemirror/language";
import { getTagCompletionOptions, tagCompletions } from "./autocomplete";

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

/**
 * @param {import("@codemirror/autocomplete").CompletionContext} context
 *
 * @returns {import("@codemirror/autocomplete").CompletionResult | undefined}
 */
function viewLanguageComplete(context) {
  let nodeBefore = syntaxTree(context.state).resolveInner(context.pos, -1);

  if (nodeBefore.name === "Ident") nodeBefore = nodeBefore.prevSibling;

  if (
    nodeBefore &&
    context.state.sliceDoc(nodeBefore.from, nodeBefore.to) === "'"
  ) {
    nodeBefore = nodeBefore.prevSibling;
  }

  if (nodeBefore?.name === "@") {
    const options = getTagCompletionOptions(context.state);
    return {
      from: nodeBefore.to,
      options,
      validFor: /@'?\w*/,
    };
  }
}

const viewLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [viewHighlight],
  }),
});

export const viewLanguageExtension = [
  viewLanguage,
  syntaxHighlighting(viewHighlightStyle),
  tagCompletions,
  viewLanguage.data.of({ autocomplete: viewLanguageComplete }),
  autocompletion(),
];
