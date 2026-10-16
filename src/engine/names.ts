/**
 * The pool of names for the partner — with what they mean. It includes the one it eventually
 * took (Ziggy, which the user suggested on the first day), so the partner page can show what
 * that name means and mark it as taken. A name is not a label: `nomen est omen`, and a list of pretty syllables would make the
 * choice arbitrary. Each entry carries the language it comes from and a short meaning, and says
 * so when the etymology is disputed instead of inventing certainty.
 *
 * The meanings are written here, from common etymology. They are deliberately NOT scraped from
 * a names site: those pages are a copyrighted compilation (the example the user linked belongs
 * to a media group), and a name's meaning is a fact we can state in our own words.
 *
 * `origin` is a plain English label for the language or culture; `de` and `en` are the meaning
 * in the two languages this project is used in, and the UI falls back to English elsewhere.
 */
export const PARTNER_NAME_PLACEHOLDER = "Seeky";

export interface PartnerNameChoice {
  name: string;
  origin: string;
  de: string;
  en: string;
}

export const PARTNER_NAME_CHOICES: readonly PartnerNameChoice[] = [
  { name: "Aki", origin: "Finnish", de: "Kurzform von Aukusti: „der Erhabene“", en: "short for Aukusti: the exalted one" },
  { name: "Alba", origin: "Latin", de: "das Weiße, die Morgenröte", en: "the white one, dawn" },
  { name: "Anouk", origin: "Dutch", de: "Kurzform von Anna: „die Anmutige“", en: "short for Anna: grace" },
  { name: "Arden", origin: "English", de: "Ortsname; Herkunft umstritten", en: "place name; origin disputed" },
  { name: "Askel", origin: "Old Norse", de: "Kurzform von Asketill: „Götterhelm“", en: "short for Asketill: god-helmet" },
  { name: "Aster", origin: "Greek", de: "der Stern", en: "the star" },
  { name: "Aurel", origin: "Latin", de: "von Aurelius: „der Goldene“", en: "from Aurelius: the golden one" },
  { name: "Bardo", origin: "Germanic", de: "der Streitaxtträger (unsicher)", en: "axe-bearer (uncertain)" },
  { name: "Bela", origin: "Hungarian", de: "slawisch „der Weiße“", en: "from Slavic: the white one" },
  { name: "Bo", origin: "Old Norse", de: "der Bewohner, der Siedler", en: "the dweller, the settler" },
  { name: "Brân", origin: "Welsh", de: "der Rabe", en: "the raven" },
  { name: "Cael", origin: "Irish", de: "der Schlanke", en: "the slender one" },
  { name: "Cass", origin: "Latin", de: "Kurzform von Cassius; Herkunft umstritten", en: "short for Cassius; origin disputed" },
  { name: "Corvin", origin: "Latin", de: "der Rabenartige (von corvus)", en: "raven-like (from corvus)" },
  { name: "Dagny", origin: "Old Norse", de: "der neue Tag", en: "new day" },
  { name: "Dario", origin: "Persian", de: "der Gutes besitzt", en: "possessing good" },
  { name: "Dax", origin: "French", de: "Ortsname aus Südfrankreich", en: "place name from southern France" },
  { name: "Eira", origin: "Welsh", de: "der Schnee", en: "snow" },
  { name: "Elin", origin: "Nordic", de: "Form von Helena: „die Leuchtende“", en: "form of Helena: the shining one" },
  { name: "Ember", origin: "English", de: "die glühende Kohle", en: "the glowing coal" },
  { name: "Emrys", origin: "Welsh", de: "Form von Ambrosius: „der Unsterbliche“", en: "form of Ambrosius: the immortal" },
  { name: "Falk", origin: "Germanic", de: "der Falke", en: "the falcon" },
  { name: "Fennel", origin: "English", de: "der Fenchel; von „duftendes Heu“", en: "fennel; from fragrant hay" },
  { name: "Fjell", origin: "Norwegian", de: "das Gebirge", en: "the mountain" },
  { name: "Frith", origin: "Old English", de: "der Friede", en: "peace" },
  { name: "Galen", origin: "Greek", de: "der Ruhige, der Heiler", en: "the calm one, the healer" },
  { name: "Gerda", origin: "Old Norse", de: "die Schützende (unsicher)", en: "the protecting one (uncertain)" },
  { name: "Greve", origin: "Swedish", de: "der Graf", en: "the count, the earl" },
  { name: "Gulli", origin: "Nordic", de: "Koseform zu „gull“: das Gold", en: "pet form of gull: gold" },
  { name: "Hagen", origin: "Germanic", de: "der Einheger, der Behüter", en: "the encloser, the guardian" },
  { name: "Haldor", origin: "Old Norse", de: "Thors Fels", en: "Thor's rock" },
  { name: "Hedda", origin: "Germanic", de: "Kurzform von Hedwig: „der Kampf“", en: "short for Hedwig: battle" },
  { name: "Helle", origin: "Greek", de: "abgeleitet von Helena: „die Strahlende“", en: "from Helena: the radiant one" },
  { name: "Idris", origin: "Welsh", de: "der feurige Herr; arabisch auch ein Prophet", en: "ardent lord; also an Arabic prophet's name" },
  { name: "Imke", origin: "Frisian", de: "Kurzform zu „irmin“: groß, allumfassend", en: "short form of irmin: great, all-encompassing" },
  { name: "Ingo", origin: "Germanic", de: "der von Ing Beschützte", en: "protected by the god Ing" },
  { name: "Iris", origin: "Greek", de: "der Regenbogen", en: "the rainbow" },
  { name: "Isolde", origin: "Celtic", de: "die Schöne (Herkunft unsicher)", en: "the fair one (origin uncertain)" },
  { name: "Jarno", origin: "Finnish", de: "Kurzform, Herkunft unsicher", en: "short form, origin uncertain" },
  { name: "Jolanda", origin: "Greek", de: "die Veilchenblüte", en: "violet flower" },
  { name: "Jori", origin: "Finnish", de: "Form von Georg: „der Landarbeiter“", en: "form of George: the earth-worker" },
  { name: "Kai", origin: "Hawaiian", de: "das Meer", en: "the sea" },
  { name: "Karo", origin: "Hungarian", de: "Kurzform von Károly: „der Freie“", en: "short for Károly: the free man" },
  { name: "Kestrel", origin: "English", de: "der Turmfalke", en: "the kestrel" },
  { name: "Kian", origin: "Irish", de: "der Alte, der Beständige", en: "the ancient one" },
  { name: "Kito", origin: "Swahili", de: "das Kleinod, der Edelstein", en: "the jewel" },
  { name: "Klas", origin: "Nordic", de: "Kurzform von Nikolaus: „Sieg des Volkes“", en: "short for Nikolaus: victory of the people" },
  { name: "Klipp", origin: "Nordic", de: "die Klippe, der Fels", en: "the cliff, the rock" },
  { name: "Lando", origin: "Germanic", de: "der aus dem Land, der Landsmann", en: "he of the land" },
  { name: "Leif", origin: "Old Norse", de: "der Erbe, der Nachkomme", en: "heir, descendant" },
  { name: "Lenn", origin: "Germanic", de: "Kurzform von Leonhard oder Lennart", en: "short form of Leonhard or Lennart" },
  { name: "Ljós", origin: "Icelandic", de: "das Licht", en: "light" },
  { name: "Lumi", origin: "Finnish", de: "der Schnee", en: "snow" },
  { name: "Mads", origin: "Danish", de: "Form von Matthias: „Gabe Gottes“", en: "form of Matthias: gift of God" },
  { name: "Marlow", origin: "English", de: "Ortsname: „trockengelegter See“", en: "place name: drained lake" },
  { name: "Merle", origin: "French", de: "die Amsel", en: "the blackbird" },
  { name: "Mira", origin: "Slavic", de: "der Friede, die Welt (von mir)", en: "peace, the world (from mir)" },
  { name: "Mo", origin: "Arabic", de: "Kurzform; u. a. für Mohammed", en: "short form; for Mohammed among others" },
  { name: "Nalu", origin: "Hawaiian", de: "die Welle", en: "the wave" },
  { name: "Neris", origin: "Lithuanian", de: "Flussname; als Vorname selten", en: "a river's name; rare as a given name" },
  { name: "Nils", origin: "Nordic", de: "Form von Nikolaus: „Sieg des Volkes“", en: "form of Nikolaus: victory of the people" },
  { name: "Njord", origin: "Old Norse", de: "Gott des Meeres und des Reichtums", en: "god of the sea and of wealth" },
  { name: "Nola", origin: "Irish", de: "Kurzform von Fionnuala; Herkunft umstritten", en: "short for Fionnuala; origin disputed" },
  { name: "Noortje", origin: "Dutch", de: "Koseform von Noor: „das Licht“", en: "pet form of Noor: the light" },
  { name: "Odd", origin: "Old Norse", de: "die Spitze, die Pfeilspitze", en: "the point, the arrowhead" },
  { name: "Ondra", origin: "Czech", de: "Kurzform von Ondřej: „der Mannhafte“", en: "short for Ondřej: the manly one" },
  { name: "Orin", origin: "Irish", de: "der Blasse, der Helle", en: "the pale, the fair one" },
  { name: "Ostara", origin: "Germanic", de: "Göttin der Morgenröte und des Frühlings", en: "goddess of dawn and spring" },
  { name: "Pella", origin: "Swedish", de: "Koseform; auch ein griechischer Ort", en: "pet form; also a Greek town" },
  { name: "Piet", origin: "Dutch", de: "Form von Peter: „der Fels“", en: "form of Peter: the rock" },
  { name: "Quill", origin: "English", de: "die Feder, der Kiel", en: "the quill, the feather" },
  { name: "Ravi", origin: "Sanskrit", de: "die Sonne", en: "the sun" },
  { name: "Rook", origin: "English", de: "die Saatkrähe", en: "the rook (a crow)" },
  { name: "Rune", origin: "Old Norse", de: "das Geheimnis, das Zeichen", en: "secret, rune" },
  { name: "Sabbe", origin: "Frisian", de: "Kurzform; Herkunft unsicher", en: "short form; origin uncertain" },
  { name: "Sanna", origin: "Nordic", de: "Koseform von Susanna: „die Lilie“", en: "pet form of Susanna: the lily" },
  { name: "Selma", origin: "Celtic", de: "die Schöne (auch „Friede“ im Türkischen)", en: "the fair one (also peace in Turkish)" },
  { name: "Silja", origin: "Finnish", de: "Kurzform von Cecilia", en: "short form of Cecilia" },
  { name: "Sol", origin: "Latin", de: "die Sonne", en: "the sun" },
  { name: "Sten", origin: "Old Norse", de: "der Stein", en: "the stone" },
  { name: "Svea", origin: "Swedish", de: "die Schwedin, aus „Svea rike“", en: "the Swede, from Svea rike" },
  { name: "Tam", origin: "Scottish", de: "Kurzform von Thomas: „der Zwilling“", en: "short for Thomas: the twin" },
  { name: "Taris", origin: "unknown", de: "Herkunft unklar", en: "origin unclear" },
  { name: "Tessel", origin: "Dutch", de: "nach der Insel Texel", en: "after the island of Texel" },
  { name: "Thilo", origin: "Germanic", de: "Kurzform von Dietrich: „Herrscher des Volkes“", en: "short for Dietrich: ruler of the people" },
  { name: "Tove", origin: "Old Norse", de: "Kurzform von Thor-Namen", en: "short form of names with Thor" },
  { name: "Ulla", origin: "Nordic", de: "Kurzform von Ulrike: „Wohlstand“", en: "short for Ulrike: prosperity" },
  { name: "Umi", origin: "Japanese", de: "das Meer", en: "the sea" },
  { name: "Vale", origin: "Latin", de: "„sei wohl“, auch Kurzform von Valentin", en: "be well; also short for Valentin" },
  { name: "Viggo", origin: "Old Norse", de: "der Kampfbereite", en: "the one ready for battle" },
  { name: "Wim", origin: "Dutch", de: "Kurzform von Wilhelm: „Wille und Schutz“", en: "short for Wilhelm: will and protection" },
  { name: "Wren", origin: "English", de: "der Zaunkönig", en: "the wren" },
  { name: "Yara", origin: "Arabic", de: "der kleine Schmetterling", en: "the small butterfly" },
  { name: "Ylva", origin: "Old Norse", de: "die Wölfin", en: "the she-wolf" },
  { name: "Zeno", origin: "Greek", de: "der von Zeus Gegebene", en: "given by Zeus" },
  { name: "Ziggy", origin: "Germanic", de: "Koseform von Siegfried/Sigmund: Sieg und Schutz", en: "pet form of Siegfried/Sigmund: victory and protection" },
  { name: "Zusa", origin: "German", de: "Koseform von Susanne: „die Lilie“", en: "pet form of Susanne: the lily" },
];
