import { computed, ref } from "vue";
import { de, type Messages } from "./de";
import { en } from "./en";
import { zhHans } from "./zh-Hans";
import { ru } from "./ru";
import { fr } from "./fr";
import { es } from "./es";
import { loadJson, saveJson } from "../lib/persist";

export type { Messages };
export type Locale = "de" | "en" | "zh-Hans" | "ru" | "fr" | "es";

type Leaves<T, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends string ? `${P}${K}` : Leaves<T[K], `${P}${K}.`>;
}[keyof T & string];

export type MessageKey = Leaves<Messages>;

const STORAGE_KEY = "ds-ide.locale";

const catalogs: Record<Locale, Messages> = { de, en, "zh-Hans": zhHans, ru, fr, es };

export const locales: readonly Locale[] = ["de", "en", "zh-Hans", "ru", "fr", "es"];

/** Language names are their own labels — that is the convention users expect. */
const languageNames: Record<Locale, string> = {
  de: "Deutsch",
  en: "English",
  "zh-Hans": "简体中文",
  ru: "Русский",
  fr: "Français",
  es: "Español",
};

export function localeLabel(locale: Locale): string {
  return languageNames[locale] ?? locale;
}

function isLocale(value: unknown): value is Locale {
  return typeof value === "string" && (locales as readonly string[]).includes(value);
}

function detectLocale(): Locale {
  const stored = loadJson<unknown>(STORAGE_KEY, null);
  if (isLocale(stored)) return stored;
  const nav = typeof navigator === "undefined" ? "en" : navigator.language.toLowerCase();
  if (nav.startsWith("de")) return "de";
  if (nav.startsWith("zh")) return "zh-Hans";
  if (nav.startsWith("ru")) return "ru";
  return "en";
}

export const locale = ref<Locale>(detectLocale());

export function setLocale(next: Locale): void {
  locale.value = next;
  saveJson(STORAGE_KEY, next);
  if (typeof document !== "undefined") document.documentElement.lang = next;
}

function resolve(catalog: Messages, key: string): string | undefined {
  let node: unknown = catalog;
  for (const part of key.split(".")) {
    if (typeof node !== "object" || node === null) return undefined;
    node = (node as Record<string, unknown>)[part];
  }
  return typeof node === "string" ? node : undefined;
}

export function t(key: MessageKey, params?: Record<string, string | number>): string {
  const raw = resolve(catalogs[locale.value], key) ?? resolve(de, key) ?? key;
  if (!params) return raw;
  return raw.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in params ? String(params[name]) : match,
  );
}

export function useI18n() {
  return {
    t,
    locale,
    setLocale,
    locales,
    isCurrent: computed(() => locale.value),
  };
}
