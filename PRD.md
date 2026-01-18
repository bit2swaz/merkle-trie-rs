# Product Requirements Document: `merkle-trie-rs`

**Project Name:** `merkle-trie-rs`
**Version:** 1.0.0
**Status:** Complete
**Classification:** Ethereum Core / Cryptography / Data Structures

---

## 1. Executive Summary

**`merkle-trie-rs`** is a high-fidelity implementation of the **Modified Merkle Patricia Trie (MPT)** as defined in the Ethereum Yellow Paper (Appendix D). The primary objective is to simulate the state storage mechanism of the Ethereum Virtual Machine (EVM).

Unlike generic binary trees, this project implements a **hexary radix tree** with cryptographic commitment. It allows for the storage of arbitrary Key-Value pairs, where the "Key" is a path through the tree and the "Value" is RLP-encoded data. The system guarantees that any modification to the data results in a deterministic change to the **State Root Hash**, providing O(1) verification of the entire dataset.

This implementation prioritizes strict specification compliance, educational depth regarding recursive ownership in Rust, and low-level manual bit manipulation.

**Project Status:** All phases complete, all success criteria verified, 64 passing tests.

---

## 2. Theoretical Framework

### 2.1 The Modified Merkle Patricia Trie

The MPT solves the inefficiency of standard Merkle Trees (which require re-hashing the whole tree for updates) and standard Patricia Tries (which are inefficient for cryptographic paths) by combining them.

The tree is composed of nodes linked deterministically. The integrity of the tree is defined by the root hash:


### 2.2 Key Encoding (Nibbles)

Ethereum keys (typically 20-byte addresses or 32-byte hashes) are treated as a sequence of **Nibbles** (4-bit units).

* **Raw Key:** `[0x12, 0x34]` (2 bytes)
* **Nibble Path:** `[1, 2, 3, 4]` (4 nibbles)

### 2.3 Hex-Prefix (HP) Encoding

To distinguish between Leaf nodes (terminators) and Extension nodes (path segments), and to handle odd-length paths, the project implements **Hex-Prefix Encoding** (compact encoding).

* **Flag 0:** Extension Node, Even path length.
* **Flag 1:** Extension Node, Odd path length.
* **Flag 2:** Leaf Node, Even path length.
* **Flag 3:** Leaf Node, Odd path length.

---

## 3. Data Structure Specification

The core logic relies on a recursive `enum` representing the four valid Ethereum node types.

### 3.1 The `Node` Enum

Strictly typed to represent the state of a branch.

```rust
pub enum Node {
    /// Represents an empty branch. Serializes to an empty byte string.
    Null,

    /// A node with up to 16 children + 1 value.
    /// Used when a path diverges.
    /// Children are indices [0..15] corresponding to hex chars [0..f].
    /// The 17th element is the value (if the key terminates at this branch).
    Branch {
        children: [Box<Node>; 16], // Fixed size array of recursive pointers
        value: Option<Vec<u8>>,    // Value if path ends here
    },

    /// An optimization node. Collapses a long path into a single node.
    /// Contains a "Shared Path" (nibbles) and the Next Node.
    Extension {
        prefix: Vec<u8>, // Nibbles
        next: Box<Node>, // Pointer to the next node
    },

    /// The end of a path.
    /// Contains the remainder of the path (nibbles) and the stored Value.
    Leaf {
        key_remainder: Vec<u8>, // Nibbles
        value: Vec<u8>,         // The actual stored data
    },
}

```

### 3.2 Serialization Requirements

All nodes must be serialized using **RLP (Recursive Length Prefix)** before hashing.

* **Dependency:** `rlp` crate.
* **Constraint:** You cannot simply `#[derive(RlpEncodable)]` on the Enum because the logic for encoding a `Branch` vs a `Leaf` is custom (e.g., Branch is a list of 17 items, Leaf is a list of 2 items). You must implement the `Encodable` trait manually.

---

## 4. Functional Requirements

### 4.1 Nibble Manipulation (Manual Implementation)

The system must interact with keys at the nibble level.

* **`to_nibbles(bytes: &[u8]) -> Vec<u8>`**: Splits `0xAF` into `[0xA, 0xF]`.
* **`from_nibbles(nibbles: &[u8]) -> Vec<u8>`**: Recombines nibbles into bytes.
* **`get_common_prefix(a, b)`**: Returns the shared nibbles between two keys (essential for splitting Extension nodes).

### 4.2 Insertion Logic (`insert`)

The most complex logic. When inserting a Key-Value pair:

1. **Traverse** the current node path matching nibbles.
2. **Case: Null Node**: Create a new Leaf.
3. **Case: Leaf Node**:
* If paths match exactly: Update value.
* If paths diverge: Convert existing Leaf to an Extension/Branch split.


4. **Case: Branch Node**: Recurse into the child index corresponding to the next nibble.
5. **Case: Extension Node**:
* If partial match: Split extension into `Extension -> Branch -> (Extension + Leaf)`.


6. **Recalculate Hashes**: As the recursion unwinds, the hashes of modified nodes change.

### 4.3 Root Calculation

* Expose a method `root_hash(&self) -> [u8; 32]`.
* This triggers a recursive RLP encoding + Keccak256 hashing of the top node.

### 4.4 Merkle Proof Generation (`get_proof`)

This is the "Light Client" requirement.

* **Input:** `key` (String or Bytes).
* **Output:** `Vec<Vec<u8>>` (A list of RLP-encoded nodes).
* **Logic:**
1. Traverse from Root to Leaf for the given key.
2. At every step, capture the RLP raw data of the current node.
3. Return the vector of these nodes.


* **Verification (Mental Check):** A client should be able to hash the first item to get the Root, and hash subsequent items to prove they are children of the previous item.

---

## 5. Technical Implementation Details

### 5.1 Technology Stack

* **Language:** Rust (Stable)
* **Cryptographic Hash:** `tiny-keccak` (Keccak256 variants).
* **Serialization:** `rlp` (Ethereum standard serialization).
* **Visualization:** `hex` (For displaying hashes).
* **Error Handling:** `thiserror` (Custom errors for "Path Not Found", "Invalid Nibble", etc.).

### 5.2 Module Architecture

```text
src/
├── lib.rs          # Public API
├── nibbles.rs      # Nibble struct and bitwise manipulation logic
├── node.rs         # Node Enum and RLP implementation
├── trie.rs         # Core recursive insert/get/proof logic
└── utils.rs        # Hex prefix encoding helpers

```

### 5.3 Memory Model

* **Single Threaded:** The Trie owns its data. No `Arc<Mutex<>>`.
* **Ownership:** The `Trie` struct owns the Root `Node`.
* **Recursion:** `Box<Node>` allows the compiler to size the recursive enum.

---

## 6. Interface Specification (CLI)

The project will expose a CLI using `clap`.

### Commands

**1. Insert**

```bash
$ merkle-trie-rs insert <KEY> <VALUE>
> Key: "user1" (0x7573657231)
> Value: "100"
> New State Root: 0x8a90...f2

```

**2. Get**

```bash
$ merkle-trie-rs get <KEY>
> Value: "100"

```

**3. Generate Proof**

```bash
$ merkle-trie-rs proof <KEY>
> Generating Merkle Proof for "user1"...
> Layer 0 (Root): 0xf902... (Hash: 0x8a90...)
> Layer 1: 0xc482...        (Hash: 0x12b4...)
> Layer 2 (Leaf): 0x8234... (Hash: 0x99a1...)
> Proof Valid.

```

**4. Print (ASCII Visualizer)**

* **Goal:** Human readable debug.
* **Format:**

```text
Root (Extension) [Path: 7, Next: ->]
  └── Branch
       ├── [3] -> Leaf [Path: 5, Value: "cat"]
       └── [5] -> Leaf [Path: 9, Value: "dog"]

```

---

## 7. Success Criteria & Roadmap

### Phase 1: The Primitive (Difficulty: ⭐)

* Implement `Nibbles` struct.
* Unit tests for splitting bytes into nibbles and Hex-Prefix encoding.

### Phase 2: The Structure (Difficulty: ⭐⭐)

* Implement `Node` Enum.
* Implement `rlp::Encodable` for `Node`.
* Verify that `keccak256(rlp(node))` matches Ethereum test vectors (if available) or internal consistency.

### Phase 3: The Insertion (Difficulty: ⭐⭐⭐)

* Implement recursive `insert`.
* Handle the "Split Extension" edge case (the hardest part of MPT).
* Visualizer `print` command to debug splits.

### Phase 4: The Proof (Difficulty: ⭐⭐⭐)

* Implement `get_proof`.
* Final CLI polish.
