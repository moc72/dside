/**
 * External targets for the account menu.
 *
 * These are the *only* places the app points outside itself — change them here, nothing
 * else references the URLs.
 */
export const links = {
  /** DeepSeek profile / account. */
  provider: "https://platform.deepseek.com/profile",
  /** Where DeepSeek shows usage and lets you top up credits. */
  usage: "https://platform.deepseek.com/usage",
  /** Project home page. */
  home: "https://modestcoders.eu",
  /** Source and issue tracker. */
  github: "https://github.com/moc72/dside",
  /** Bug reports and questions. */
  issues: "https://github.com/moc72/dside/issues",
  /** Direct contact. */
  mail: "mailto:me@modestcoders.eu",
} as const;
