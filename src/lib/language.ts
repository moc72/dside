const byExtension: Record<string, string> = {
  ts: "typescript",
  tsx: "typescript",
  js: "javascript",
  jsx: "javascript",
  mjs: "javascript",
  cjs: "javascript",
  json: "json",
  html: "html",
  htm: "html",
  vue: "html",
  css: "css",
  scss: "scss",
  less: "less",
  md: "markdown",
  markdown: "markdown",
  rs: "rust",
  py: "python",
  toml: "ini",
  ini: "ini",
  yml: "yaml",
  yaml: "yaml",
  sh: "shell",
  bash: "shell",
  sql: "sql",
  xml: "xml",
};

export function languageFromPath(path: string): string {
  const name = path.split("/").pop() ?? path;
  const extension = name.includes(".") ? name.split(".").pop()!.toLowerCase() : "";
  return byExtension[extension] ?? "plaintext";
}
