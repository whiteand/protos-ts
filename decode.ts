import { Reader } from "protobufjs/minimal";
import { Person } from "./types";

function decode(reader: Reader | Uint8Array, length?: number): Person {
  const r = reader instanceof Reader ? reader : Reader.create(reader);
  const end = length === undefined ? r.len : r.pos + length;
  const message: Person = {
    name: ''
  }
  while (r.pos < end) {
    const tag = r.uint32();
    switch (tag >>> 3) {
      case 1: {
        message.name = r.string();
        break;
      }
      default:
        r.skipType(tag & 7);
        break;
    }
  }
  return message;
}
