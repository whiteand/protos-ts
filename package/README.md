# protos-ts

This a CLI tool for transforming of .proto files into typescript modules. It is faster alternative to [protobufjs](https://www.npmjs.com/package/protobufjs) package.

## Install

```
# global install
npm install --global protos-ts

# local install
npm install -D protos-ts
```

## Rationale

`protobufjs` is currently used as a main javascript(and typesript) compiler. But there are several problems with this tool.

It produces single large javascript file.
It does not have typings by default.
Types are introduced as external `index.d.ts` file.

What are the results of it? It is not easily tree-shakable.

Usually the case is that some messages in your project are "encode"-only and some of the are "decode" only. But since they are highly coupled - you must load both encoding and decoding functions. This tool for each message creates separated folder with all necessary function placed in it's own place.

Also `protobufjs` uses JS to read, parse and compile `.proto` schemas to javascript. This tool is written in Rust. So as a result it is faster than [protobufjs] alternative.

# How to use

Example:

```
# if installed locally
protos-ts ./proto --out ./typescript-schemas

# via npx
npx protos-ts ./proto --out ./typescript-schemas
```

As you can see it has such structure:

```
protos-ts <input folder full of .proto files> --out <output folder>
```

`input folder full of .proto files` is a folder that contains all of your .proto files.

Example:

```
proto
 | Action.proto
 | Commons
    | Enums.proto
    | Types.proto
```

`output folder` is the folder where you want your typescript files to appear.

### Result

```
out
  | Action
    | MyMessage
      | decode.ts
      | encode.ts
  | Commons
    | Enums
      | MyEnum.ts
    | types.ts
```

## Performance

I didn't do it properly. But on my computer I've compared this implementation with official [compiler](https://github.com/protobufjs);
I've took protocol buffer files from my work project and compiled both of them several times and took minimum.

```
This compiler: 0.9s
official: 7s
```

Actually it is difficult to compare them because they do a different job.

Official `protobuf.js` compiler generates big large file. While this compiler produces hierarchy of different files. Therefore we have much more IO operations.

I've found that in my compiler 92% of the time is spent in IO operations to the harddrive.

```
collect file names 5.328ms
parsing files 44.7105ms
compiling files to typescript 33.6335ms
saving typescript files 1.0607062s
Full Time 1.1485477s
```

