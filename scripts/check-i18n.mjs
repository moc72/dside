#!/usr/bin/env node
// Keeps the six catalogs honest: every key in `de` exists everywhere else, no catalog has
// keys the reference lacks, and no translation still carries the German text. The TypeScript
// types already reject a missing key at build time; this script also runs without a build,
// finds duplicate keys (the type checker silently keeps the last one) and reports values that
// merely repeat the German source.
import { readFileSync } from "node:fs";
import ts from "typescript";

const FILES = {
  de: "src/i18n/de.ts",
  en: "src/i18n/en.ts",
  es: "src/i18n/es.ts",
  fr: "src/i18n/fr.ts",
  ru: "src/i18n/ru.ts",
  "zh-Hans": "src/i18n/zh-Hans.ts",
};

/** The object literal assigned to `export const <name> = { ... }`. */
function catalogObject(source, file) {
  const text = readFileSync(file, "utf8");
  const tree = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  let found = null;
  const visit = (node) => {
    if (ts.isVariableStatement(node)) {
      for (const declaration of node.declarationList.declarations) {
        if (
          ts.isIdentifier(declaration.name) &&
          declaration.initializer &&
          ts.isObjectLiteralExpression(declaration.initializer)
        ) {
          found = declaration.initializer;
          return;
        }
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(tree);
  if (!found) throw new Error(`${source}: no catalog object literal found`);
  return found;
}

function leaves(node, prefix = "", out = new Map(), duplicates = []) {
  for (const property of node.properties) {
    if (!ts.isPropertyAssignment(property)) continue;
    const name = property.name.getText().replace(/^["']|["']$/g, "");
    const path = prefix ? `${prefix}.${name}` : name;
    const value = property.initializer;
    if (ts.isObjectLiteralExpression(value)) {
      leaves(value, path, out, duplicates);
    } else if (ts.isStringLiteral(value) || ts.isNoSubstitutionTemplateLiteral(value)) {
      if (out.has(path)) duplicates.push(path);
      out.set(path, value.text);
    }
  }
  return { values: out, duplicates };
}

const catalogs = {};
const problems = [];
const warnings = [];
for (const [locale, file] of Object.entries(FILES)) {
  const node = catalogObject(locale, file);
  const { values, duplicates } = leaves(node);
  catalogs[locale] = values;
  for (const duplicate of duplicates) problems.push(`${file}: duplicate key "${duplicate}"`);
}

const reference = catalogs.de;
for (const [locale, values] of Object.entries(catalogs)) {
  if (locale === "de") continue;
  const file = FILES[locale];
  for (const key of reference.keys()) {
    if (!values.has(key)) problems.push(`${file}: missing "${key}"`);
  }
  for (const key of values.keys()) {
    if (!reference.has(key)) problems.push(`${file}: unknown key "${key}" (not in de.ts)`);
  }
  for (const [key, value] of values) {
    const german = reference.get(key);
    // Same text as the source and German-looking: either untranslated or a word that is
    // legitimately identical (product names, "MIT", "Skills"). The latter is only a warning.
    if (german === value && /[äöüßÄÖÜ]|\b(der|die|das|und|nicht|wird|kann|dein|deine|für)\b/.test(value)) {
      warnings.push(`${file}: "${key}" is identical to the German text`);
    }
  }
}

console.log(
  `i18n: ${reference.size} keys, ${Object.keys(catalogs).length} catalogs ` +
    `(${Object.keys(catalogs).join(", ")})`,
);
for (const warning of warnings) console.warn(`warn: ${warning}`);
for (const problem of problems) console.error(`error: ${problem}`);
if (problems.length) {
  console.error(`i18n: ${problems.length} problem(s)`);
  process.exit(1);
}
console.log("i18n: ok");
