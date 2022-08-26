# protobufts [WIP]

This a CLI tool for transforming of .proto files into typescript modules.

It will be at least 4 times faster than current best compiler.

## Rationale

`protobufjs` is currently used as a main compiler. But there are several problems with this tool.

It produces single large javascript file.
It does not have typings by default.
Types are introduced as external `index.d.ts` file.

What are the results of it? It is not easily tree-shakable.

Usually the case is that some messages in your project are "encode" only and some of the are "decode" only. But since they are highly coupled - you must load both encoding and decoding functions. This tool for each message creates separated folder with each necessary function placed in it's own place.

Also `protobufjs` uses JS to read, parse and compile `.proto` schemas to javascript. This tool is written in Rust. So as a result it will be much faster than protobufjs.

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
  | Action
    | MyMessages
      | decode.ts
      | encode.ts
  | Commons
    | Enums.ts
      | MyEnum
        | decode.ts
        | encode.ts
    | Types.ts
    | index.ts
```

## TODOs

| Development Task                  | Progress      |
| :-------------------------------- | :------------ |
| Lexical Analyzer                  | **Done**      |
| Syntactical Analyzer              | **Done**      |
| Proto Packages -> Package Tree    | **Done**      |
| Create name resolution mechanism  | **Done**      |
| Generation of interfaces          | **Done**      |
| Encoding of basic types           | **Done**      |
| Encoding of repeated basic types  | **Done**      |
| Encoding of enums                 | **Done**      |
| Encoding of user defined messages | **Done**      |
| Fix resolving algorithm           | **Done**      |
| Encoding of oneof messages        | **Done**      |
| Encoding of google defined        | *In Progress* |
| Decoding of basic types           | Open          |
| Decoding of enums                 | Open          |
| Decoding of user defined messages | Open          |
| Full Coverage Tests               | Open          |
| Reach CLI experience              | Open          |


