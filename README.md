# protobufts

Is an executable that takes folder with .proto files in it and creates Typescript module (with nested submodules).

## Usage

```
protobufts ./proto --out ./out
```

`./proto`

```
proto
 | Action.proto
 | Commons
    | Enums.proto
    | Types.proto
```

### Result

```
out
  | Action.ts
  | Commons
    | Enums.ts
    | Types.ts
    | index.ts
```

## Roadmap

| Development Task               | Progress      |
| :----------------------------- | :------------ |
| Lexical Analyzer               | **Done**      |
| Syntactical Analyzer           | **Done**      |
| Proto Packages -> Package Tree | **Done**      |
| Package Tree -> TS Ast         | *In Progress* |
| TS Ast -> String               | Blocked       |
| decode                         | Blocked       |
| encode                         | Blocked       |
| Tests                          | Blocked       |


