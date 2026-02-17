---
theme: seriph
layout: cover
title: gaji
class: text-center
transition: slide-left
fonts:
  sans: RidiBatang
  mono: D2Coding
  provider: none
---

<img src="/logo.png" class="mx-auto w-40" />

# gaji

TypeScript로 작성하는 GitHub Actions Workflow

---

# 샘플 슬라이드

테마 확인용 슬라이드입니다.

- 항목 하나
- 항목 둘
- 항목 셋

---

# 코드 블록 확인

```ts
import { getAction } from "gaji";

const checkout = getAction("actions/checkout@v4");

const job = new Job("build", {
  runsOn: "ubuntu-latest",
  steps: [checkout()],
});
```

---
layout: center
class: text-center
---

# 감사합니다
