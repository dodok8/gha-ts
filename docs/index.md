---
layout: home

hero:
  name: gaji
  text: Type-safe GitHub Actions
  tagline: Write GitHub Actions workflows in TypeScript with full type safety
  image:
    src: /logo.png
    alt: gaji
  actions:
    - theme: brand
      text: Why gaji?
      link: /guide/why
    - theme: alt
      text: View on GitHub
      link: https://github.com/dodok8/gaji

features:
  - icon: 🔒
    title: Full Type Safety
    details: Know every action input and whether it's required through types.

  - icon: ⚡
    title: Fast Development
    details: File watching automatically fetches action.yml and generates types. Change your workflow and see updated types immediately.

  - icon: 📦
    title: Works Everywhere
    details: Use with any language or build tool without restructuring your project. Bundles QuickJS internally — no Node.js or external JS runtime required.

  - icon: 🔄
    title: Easy Migration
    details: Migrate existing YAML workflows to TypeScript with a single command.
---
