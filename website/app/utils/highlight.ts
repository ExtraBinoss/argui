import { createHighlighterCore } from 'shiki/core'
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript'
import rust from 'shiki/langs/rust.mjs'
import shellscript from 'shiki/langs/shellscript.mjs'
import toml from 'shiki/langs/toml.mjs'
import githubDark from 'shiki/themes/github-dark.mjs'
import githubLight from 'shiki/themes/github-light.mjs'

const highlighter = createHighlighterCore({
  themes: [githubLight, githubDark],
  langs: [rust, shellscript, toml],
  engine: createJavaScriptRegexEngine(),
})

export async function highlightCode(code: string, language: 'rust' | 'shellscript' | 'toml') {
  return (await highlighter).codeToHtml(code, {
    lang: language,
    themes: { light: 'github-light', dark: 'github-dark' },
    defaultColor: false,
  })
}
