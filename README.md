# TCS
(Tape Canonical Serialization)

A high-performance, schema-based binary serialization format for Tapedrive.

TCS generates Rust code from `.tcs` schema files, producing types with efficient binary serialization. The encoding is canonical (deterministic), making it suitable for cryptographic hashing and blockchain applications.

## Features

- **Schema-driven**: Define your data structures in `.tcs` files
- **Canonical encoding**: Deterministic byte output for cryptographic applications
- **High performance**: 20-60x faster than BCS in benchmarks
- **Fixed-size arrays**: Native support for `byte[32]` hash fields
- **Zero-copy deserialization**: Placement initialization for maximum speed

## Background

TCS is derived from [Kiwi](https://github.com/evanw/kiwi), a binary serialization format created by [Evan Wallace](https://github.com/evanw) for Figma's multiplayer syncing and file storage. The schema syntax is intentionally similar.

Unlike Protocol Buffers, TCS is non-self-describing—the schema is not embedded in the data stream, resulting in more compact output. This is ideal when both endpoints already know the schema (as in Tapedrive's node communication). TCS differs from Kiwi by using fixed-width little-endian integers instead of varints, ensuring canonical output for cryptographic hashing.

## Quick Start

### 1. Define a Schema

Create a `schema.tcs` file:

```proto
package myapp;

enum Status {
    PENDING = 1;
    ACTIVE = 2;
    COMPLETED = 3;
}

struct Header {
    uint64 height;
    byte[32] hash;
    uint64 timestamp;
}

message Transaction {
    byte[32] txHash = 1;
    uint64 nonce = 2;
    byte[] payload = 3;
}
```

### 2. Generate Rust Code

```bash
tcs gen-rust --input schema.tcs --output generated.rs
```

### 3. Use Generated Types

```rust
use myapp::{Header, Transaction, Status};

// Serialize
let header = Header {
    height: 100,
    hash: [0u8; 32],
    timestamp: 1234567890,
};
let bytes = wincode::serialize(&header).unwrap();

// Deserialize
let decoded: Header = wincode::deserialize(&bytes).unwrap();
```

## Schema Syntax

### Types

| TCS Type   | Rust Type   | Description                    |
|------------|-------------|--------------------------------|
| `bool`     | `bool`      | Boolean (1 byte)               |
| `byte`     | `u8`        | Unsigned 8-bit integer         |
| `int`      | `i32`       | Signed 32-bit integer          |
| `uint`     | `u32`       | Unsigned 32-bit integer        |
| `int64`    | `i64`       | Signed 64-bit integer          |
| `uint64`   | `u64`       | Unsigned 64-bit integer        |
| `float`    | `f32`       | 32-bit float (avoid for canonical) |
| `string`   | `String`    | UTF-8 string                   |
| `byte[N]`  | `[u8; N]`   | Fixed-size byte array          |
| `T[]`      | `Vec<T>`    | Variable-length array          |

### Definitions

**Enums** - Fixed set of values with explicit discriminants:
```
enum NodeRole {
    STORAGE = 1;
    VALIDATOR = 2;
    LIGHT = 3;
}
```

**Structs** - Fixed fields, all required:
```
struct BlockHeader {
    uint64 height;
    byte[32] prevHash;
    byte[32] merkleRoot;
    uint64 timestamp;
}
```

**Messages** - Fields with IDs, all optional (for schema evolution):
```
message Transaction {
    byte[32] txHash = 1;
    uint64 nonce = 2;
    byte[] payload = 3;
}
```

## CLI Commands

```bash
# Generate Rust code from schema
tcs gen-rust --input schema.tcs --output generated.rs

# Validate a schema file
tcs validate --input schema.tcs
```

## Performance

TCS significantly outperforms BCS (Binary Canonical Serialization) used in blockchain systems like Aptos and Sui.

### Serialization Speed

| Data Type              | BCS       | TCS       | Speedup      |
|------------------------|-----------|-----------|--------------|
| BlockHeader (120B)     | 477 ns    | 19 ns     | **25x faster** |
| Transaction (64B)      | 504 ns    | 32 ns     | **16x faster** |
| Transaction (256B)     | 1.03 µs   | 42 ns     | **25x faster** |
| Transaction (1KB)      | 1.89 µs   | 107 ns    | **18x faster** |
| Transaction (4KB)      | 5.31 µs   | 130 ns    | **41x faster** |

### Deserialization Speed

| Data Type              | BCS       | TCS       | Speedup      |
|------------------------|-----------|-----------|--------------|
| BlockHeader (120B)     | 544 ns    | 27 ns     | **20x faster** |
| Transaction (64B)      | 809 ns    | 49 ns     | **17x faster** |
| Transaction (256B)     | 1.46 µs   | 63 ns     | **23x faster** |
| Transaction (1KB)      | 4.63 µs   | 80 ns     | **58x faster** |
| Transaction (4KB)      | 16.4 µs   | 264 ns    | **62x faster** |

### Throughput

| Operation              | BCS          | TCS           |
|------------------------|--------------|---------------|
| Serialize (4KB tx)     | 736 MiB/s    | 29.3 GiB/s    |
| Deserialize (4KB tx)   | 238 MiB/s    | 14.4 GiB/s    |

### Serialized Sizes

| Data Type              | BCS         | TCS         | Postcard    |
|------------------------|-------------|-------------|-------------|
| BlockHeader            | 120 bytes   | 120 bytes   | 126 bytes   |
| Transaction (256B)     | 363 bytes   | 372 bytes   | 365 bytes   |
| 100 Transactions       | 36,317 bytes| 37,224 bytes| 36,470 bytes|

TCS produces ~2-3% larger output due to fixed-width length prefixes, but this is offset by dramatically faster serialization.

## TCS vs BCS Encoding

| Aspect              | BCS                              | TCS                               |
|---------------------|----------------------------------|-----------------------------------|
| Canonical           | Yes                              | Yes                               |
| Integer encoding    | Fixed-width little-endian        | Fixed-width little-endian         |
| Sequence length     | ULEB128 (variable)               | u64 (fixed 8 bytes)               |
| Optional fields     | 0x00/0x01 prefix                 | Field ID prefix                   |
| Primary use case    | Blockchain (Aptos, Sui)          | Tapedrive                         |

Both formats produce deterministic output suitable for cryptographic hashing.

## License

MIT
