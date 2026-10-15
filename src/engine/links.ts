/**
 * External targets for the account menu.
 *
 * These are the *only* places the app points outside itself — change them here, nothing
 * else references the URLs. `deepvibe.eu` and the GitHub org come from the project owner;
 * `issues` currently falls back to the org page, so point it at the real repository once
 * that exists. DeepSeek paths are the public platform routes.
 */
export const links = {
  /** DeepSeek account overview. */
  provider: "https://platform.deepseek.com",
  /** Where DeepSeek shows usage and lets you top up credits. */
  usage: "https://platform.deepseek.com/usage",
  /** Project home page. */
  home: "https://deepvibe.eu",
  /** Source / community. */
  github: "https://github.com/deepvibe-eu",
  /** Bug reports and questions. */
  issues: "https://github.com/deepvibe-eu",
} as const;
