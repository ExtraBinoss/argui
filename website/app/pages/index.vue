<script setup lang="ts">
import { useShowcaseStore } from '~/stores/showcase'

const showcase = useShowcaseStore()
useSeoMeta({
  title: 'Native interfaces, one Rust engine',
  description: 'Build native desktop and mobile interfaces with Argui. Write Solid or React TSX and render with a retained Rust engine, including a WebGPU browser preview.',
})
</script>

<template>
  <main id="main-content">
    <section class="hero shell">
      <div class="hero-copy">
        <div class="eyebrow"><span class="eyebrow-dot" /> Open source Rust UI engine <span class="eyebrow-version">v0.4</span></div>
        <h1>Build interfaces<br>that feel <em>at home.</em></h1>
        <p class="hero-lead">Write familiar Solid or React TSX. Argui keeps layout, input, animation, and rendering in one native Rust engine.</p>
        <div class="hero-actions">
          <NuxtLink class="button button-primary" to="/components">Explore components <span aria-hidden="true">↗</span></NuxtLink>
          <NuxtLink class="button button-outline" to="/docs/getting-started">Get started <span aria-hidden="true">→</span></NuxtLink>
        </div>
        <div class="hero-footnote"><span class="footnote-line" /> Desktop · Mobile · WebGPU preview</div>
      </div>
      <div class="hero-art" aria-label="Argui Solid TSX example">
        <div class="code-window">
          <div class="code-window-head"><span class="code-tab"><span class="code-tab-dot" /> counter.tsx</span><span class="code-language">Solid TSX</span></div>
          <pre><code><span class="code-purple">import</span> { createSignal } <span class="code-purple">from</span> <span class="code-green">'@argui/solid'</span>
<span class="code-purple">import</span> { Button } <span class="code-purple">from</span> <span class="code-green">'@argui/widgets/solid'</span>

<span class="code-purple">const</span> [count, setCount] = createSignal(<span class="code-orange">0</span>)

<span class="code-purple">export function</span> <span class="code-blue">Counter</span>() {
  <span class="code-purple">return</span> (
    &lt;<span class="code-red">Button</span> <span class="code-blue">onClick</span>={() =&gt; setCount(count() + <span class="code-orange">1</span>)}&gt;
      Count: {count()}
    &lt;/<span class="code-red">Button</span>&gt;
  )
}</code></pre>
        </div>
        <div class="hero-art-caption"><span class="caption-icon">↳</span> TSX describes the tree. Rust owns the interaction.</div>
      </div>
    </section>

    <section class="capability-strip">
      <div class="shell capability-grid">
        <div><span class="capability-number">01</span><strong>One retained engine</strong><p>Layout and paint stay in Rust.</p></div>
        <div><span class="capability-number">02</span><strong>Two TSX adapters</strong><p>Choose Solid or React.</p></div>
        <div><span class="capability-number">03</span><strong>Real native controls</strong><p>Focus, input, and semantics included.</p></div>
      </div>
    </section>

    <section class="preview-section shell">
      <div class="section-heading-row">
        <div><div class="section-kicker">THE COMPONENT GALLERY</div><h2>See the engine in motion.</h2><p>These controls run in the browser through the same Argui renderer compiled to WebAssembly.</p></div>
        <NuxtLink class="text-link" to="/components">View all 12 widgets <span aria-hidden="true">↗</span></NuxtLink>
      </div>
      <div class="preview-controls"><span>Live preview</span><div class="segmented" aria-label="Choose TSX adapter"><button type="button" :aria-pressed="showcase.adapter === 'solid'" @click="showcase.useAdapter('solid')">Solid</button><button type="button" :aria-pressed="showcase.adapter === 'react'" @click="showcase.useAdapter('react')">React</button></div></div>
      <GalleryFrame :adapter="showcase.adapter" :component="showcase.selected" lazy />
      <p class="preview-note">Requires WebGPU. The desktop gallery also includes native window examples.</p>
    </section>

    <section class="architecture-section">
      <div class="shell architecture-grid">
        <div><div class="section-kicker">DESIGNED AS A SYSTEM</div><h2>Simple at the edge.<br>Serious underneath.</h2><p>Compose views in TSX or use the Rust runtime directly. Both paths share the same retained tree and rendering model.</p><NuxtLink class="text-link" to="/docs/architecture">Explore the architecture <span aria-hidden="true">↗</span></NuxtLink></div>
        <div class="architecture-cards">
          <div><span class="card-icon">◇</span><h3>Native by default</h3><p>Input, focus, accessibility, layout, and animation belong to the runtime.</p></div>
          <div><span class="card-icon">⌘</span><h3>Familiar authoring</h3><p>Build the same interface with Solid or React, backed by the Argui host contract.</p></div>
          <div><span class="card-icon">◫</span><h3>Browser preview</h3><p>The Rust renderer also runs in WebAssembly on a WebGPU canvas.</p></div>
        </div>
      </div>
    </section>

    <section class="closing-cta shell"><span class="section-kicker">START BUILDING</span><h2>Your next interface starts here.</h2><p>Follow the current guides and use the gallery as a working reference.</p><div><NuxtLink class="button button-primary" to="/docs/getting-started">Read the guide <span aria-hidden="true">↗</span></NuxtLink><NuxtLink class="button button-outline" to="/components">Browse components</NuxtLink></div></section>
  </main>
</template>
