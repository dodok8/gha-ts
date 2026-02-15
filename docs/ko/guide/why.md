# 왜 gaji인가?

**gaji**는 **G**itHub **A**ctions **J**ustified **I**mprovements의 약자입니다.

한국어 "가지"에서 따왔습니다 🍆 - 채소 중에서 제일 맛있죠.

## GitHub Actions의 오류

 제가 GitHub Actions를 다루면서 느낀 단점으론 크게 3가지가 있습니다.

1. YAML은 데이터를 표시하기 위한 언어지, 입력과 출력, 사이드 이펙트가 있는 동작을 표현하기에는 적합한 언어가 아닙니다.
2. YAML에 타입 검사가 없습니다. 특히 GitHub Action는 외부 저장소에 의존할 일이 많은데(왠만한 액션의 시작인 `actions/checkout@v5` 조차도 외부 저장소 입니다.) 이들이 요구하는 입력에 대한 검증이 전혀 없습니다. 사용자가 직접 문서를 보고 일일이 형식에 맞게 입력해야 합니다.
3. 로컬에서 재현하기가 힘듭니다.

 이런 단점이 결합되어서, GitHub Actions는 실행하기 전까지는 간단한 오타 하나도 못찾는 플랫폼이 되었습니다. gaji는 이 중 첫 번째와 두 번째 단점을 해결합니다. 자동으로 외부 저장소에서 action.yml을 가져와서 타입으로 만들어줍니다.

### YAML에서 오류

```yaml
name: CI
on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v5

      - uses: actions/setup-node@v4
        with:
          node-versoin: '20'  # 키 이름 오타! 런타임까지 오류 없음 ❌
          cache: 'npm'

      - run: npm ci
      - run: npm test
```

### gaji를 이용하면...

```ts twoslash
// @filename: workflows/example.ts
// ---cut---
import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const setupNode = getAction("actions/setup-node@v4");

const build = new Job("ubuntu-latest")
  .addStep(checkout({}))
  .addStep(setupNode({
    with: {
      "node-version": "20",  // ✅ 올바른 키 이름, 컴파일 시점에 오류 포착,완전한 자동완성 및 타입 체크
      cache: "npm",  
    },
  }))
  .addStep({ run: "npm ci" })
  .addStep({ run: "npm test" });

const workflow = new Workflow({
  name: "CI",
  on: { push: { branches: ["main"] } },
}).addJob("build", build);

workflow.build("ci");
```

## Special Thanks

### gaji 브랜드

- 이름 제안: [kiwiyou](https://github.com/kiwiyou), [RanolP](https://github.com/ranolp)
- 로고 제작: [sij411](https://github.com/sij411)

### 발상

- Client Devops Team@Toss: 이 팀에서 겪은 경험이 아니었으면 YAML과 GitHub Actions에 대해 생각해보지 않았을 겁니다. 특히 아래 제품 또한 팀원의 소개를 통해 알게 되었습니다.
- [emmanuelnk/github-actions-workflow-ts](https://github.com/emmanuelnk/github-actions-workflow-ts): TS로 GiHub Actions를 표기한다는 아이디어는 여기서 가져왔습니다.
