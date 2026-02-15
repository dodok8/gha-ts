---
layout: home

hero:
  name: gaji
  text: 타입 안전한 GitHub Actions
  tagline: TypeScript로 완전한 타입 안전성을 갖춘 GitHub Actions 워크플로우 작성
  image:
    src: /logo.png
    alt: gaji
  actions:
    - theme: brand
      text: 왜 가지인가?
      link: /ko/guide/why
    - theme: alt
      text: GitHub에서 보기
      link: https://github.com/dodok8/gaji

features:
  - icon: 🔒
    title: 완전한 타입 안전성
    details: 모든 액션의 입력 및 필수 여부를 타입을 통해서 알 수 있습니다.

  - icon: ⚡
    title: 빠른 개발
    details: 파일 감시 기능으로 자동으로 action.yml을 가져오고 타입을 생성합니다. 워크플로우를 변경하면 바로 변경된 타입을 확인할 수 있습니다.

  - icon: 📦
    title: 모든 곳에서 작동
    details: 프로젝트 구조를 변경할 필요 없이 어떤 언어나 빌드 도구와도 함께 사용 가능합니다.  QuickJS를 내장하여 Node.js나 외부 JS 런타임 없이 실행 가능합니다.

  - icon: 🔄
    title: 쉬운 마이그레이션
    details: 단일 명령으로 기존 YAML 워크플로우를 TypeScript로 변경 가능합니다.
---
