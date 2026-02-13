import { defineConfig } from "vitepress";

export default defineConfig({
  title: "gaji",
  description: "Type-safe GitHub Actions workflows in TypeScript",

  head: [
    ["link", { rel: "icon", href: "/logo.png" }],
  ],

  locales: {
    root: {
      label: "English",
      lang: "en",
      themeConfig: {
        nav: [
          { text: "Guide", link: "/guide/getting-started" },
          { text: "Reference", link: "/reference/cli" },
          { text: "Examples", link: "/examples/simple-ci" },
        ],
        sidebar: {
          "/guide/": [
            {
              text: "Guide",
              items: [
                { text: "Getting Started", link: "/guide/getting-started" },
                { text: "Installation", link: "/guide/installation" },
                { text: "Writing Workflows", link: "/guide/writing-workflows" },
                { text: "Configuration", link: "/guide/configuration" },
                { text: "Migration", link: "/guide/migration" },
              ],
            },
          ],
          "/reference/": [
            {
              text: "Reference",
              items: [
                { text: "CLI Commands", link: "/reference/cli" },
                { text: "TypeScript API", link: "/reference/api" },
                { text: "Actions", link: "/reference/actions" },
              ],
            },
          ],
          "/examples/": [
            {
              text: "Examples",
              items: [
                { text: "Simple CI", link: "/examples/simple-ci" },
                { text: "Matrix Build", link: "/examples/matrix-build" },
                {
                  text: "Composite Action",
                  link: "/examples/composite-action",
                },
                {
                  text: "JavaScript Action",
                  link: "/examples/javascript-action",
                },
              ],
            },
          ],
        },
      },
    },
    ko: {
      label: "한국어",
      lang: "ko",
      link: "/ko/",
      themeConfig: {
        nav: [
          { text: "가이드", link: "/ko/guide/getting-started" },
          { text: "레퍼런스", link: "/ko/reference/cli" },
          { text: "예제", link: "/ko/examples/simple-ci" },
        ],
        sidebar: {
          "/ko/guide/": [
            {
              text: "가이드",
              items: [
                { text: "빠른 시작", link: "/ko/guide/getting-started" },
                { text: "설치", link: "/ko/guide/installation" },
                {
                  text: "워크플로우 작성",
                  link: "/ko/guide/writing-workflows",
                },
                { text: "설정", link: "/ko/guide/configuration" },
                { text: "마이그레이션", link: "/ko/guide/migration" },
              ],
            },
          ],
          "/ko/reference/": [
            {
              text: "레퍼런스",
              items: [
                { text: "CLI 명령어", link: "/ko/reference/cli" },
                { text: "TypeScript API", link: "/ko/reference/api" },
                { text: "액션", link: "/ko/reference/actions" },
              ],
            },
          ],
          "/ko/examples/": [
            {
              text: "예제",
              items: [
                { text: "간단한 CI", link: "/ko/examples/simple-ci" },
                { text: "매트릭스 빌드", link: "/ko/examples/matrix-build" },
                {
                  text: "컴포지트 액션",
                  link: "/ko/examples/composite-action",
                },
                {
                  text: "JavaScript 액션",
                  link: "/ko/examples/javascript-action",
                },
              ],
            },
          ],
        },
      },
    },
  },

  themeConfig: {
    logo: "/logo.png",
    socialLinks: [
      { icon: "github", link: "https://github.com/dodok8/gaji" },
    ],
    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2026-present",
    },
  },
});
