Chiru

A verification-first systems language for the AI era.

Chiru is a systems programming language designed to make low-level code explicit, inspectable, and trustworthy — especially when that code is written or assisted by AI.

Chiru does not replace Rust, C, or C++.
It is designed to live beside them as a verification and trust layer.

Why Chiru Exists

Systems programming is entering a new phase:

AI can generate low-level code

Humans struggle to fully trust AI-generated systems code

Existing languages rely heavily on inference and implicit rules

Safety is often assumed, not inspectable

Chiru addresses a different problem:

How do humans verify and trust systems code at scale — especially when AI is involved?

Chiru’s answer is simple and strict:

Intent must be explicit

Correctness must be inspectable

Safety must be visible

Unsafe behavior must be declared and auditable

Chiru optimizes for trust at scale, not convenience.

What Chiru Is (and Is Not)
Chiru is

A systems-level programming language

Verification-first by design

Explicit about ownership, lifetimes, and capabilities

Deterministic (no garbage collection)

Designed for AI-human collaboration

Suitable for safety-critical and infrastructure code

Chiru is not

A general-purpose application language

A framework or runtime

A replacement for Rust, C, or C++

A macro-heavy or inference-heavy language

A syntax experiment

Chiru is intentionally small, strict, and focused.

Core Principles

Chiru follows these principles strictly:

Explicit > Implicit

Verifiable > Convenient

Predictable > Powerful

Simple rules > Many rules

Trust > Popularity

Composability > Completeness

If a feature reduces auditability or increases ambiguity, Chiru rejects it.

A Small Example
function process_payment {

    lifetime payment_request bound to process_payment

    unsafe {
        assume "HSM returned a valid memory pointer"
        region external hsm_memory
        let payment_key in hsm_memory
    }

    capability SharedRead payment_key during payment_request
}

What this expresses:

Memory origin is explicit

Lifetimes are declared

Capabilities define how values may be accessed

Unsafe assumptions are named and auditable

Nothing is inferred silently

Safety Report (Human-Readable)

Every Chiru program produces a safety report.

Chiru Safety Report
==================

Summary
-------

Ownership:     VERIFIED
Lifetimes:     VERIFIED
Capabilities:  VERIFIED
Destruction:   VERIFIED

Unsafe Assumptions
------------------

[UA-001] HSM returned a valid memory pointer
  Scope: 2
  Affects: []

Values
------

1: UNSAFE (DESTROYED)

Verdict
-------

SAFE_IF_ASSUMPTIONS_HOLD

Safety Report (Machine-Readable / CI)
{
  "verdict": "SAFE_IF_ASSUMPTIONS_HOLD",
  "summary": {
    "ownership": "VERIFIED",
    "lifetimes": "VERIFIED",
    "capabilities": "VERIFIED",
    "destruction": "VERIFIED"
  }
}

Exit Codes
Exit Code Meaning
0 SAFE
1 SAFE_IF_ASSUMPTIONS_HOLD
2 UNSAFE
3 Parse / usage / IO error

This makes Chiru CI-friendly by design.

Platform Support
Prebuilt Binaries

Linux (x86_64): ✅ Officially supported

Build from Source

macOS (arm64 / x86_64): ⏳ Build from source

Windows (x86_64): ⏳ Build from source

Native binaries for macOS and Windows are planned for v0.1.1.

Installation (Linux)
curl -fsSL <https://raw.githubusercontent.com/chiru-lang/chiru/main/install.sh> | sh

After installation:

chiru --version
chiru examples/payment.chiru

If this is your first install, ensure ~/.local/bin is in your PATH.

Building from Source

Chiru is written in Rust and can be built on any platform supported by the Rust toolchain.

Requirements

Rust 1.70+ (stable)

Git

Build
git clone <https://github.com/chiru-lang/chiru.git>
cd chiru
cargo build --release

The resulting binary will be available at:

target/release/chiru

You may place it anywhere in your PATH.

Where Chiru Fits

Chiru is designed to be used inside larger systems, not to replace them.

Typical use cases:

Verifying AI-generated C or Rust FFI code

Auditing unsafe boundaries

Embedded and firmware verification

Security-critical modules (payments, crypto, HSMs)

Compliance and infrastructure trust checks

CI safety gates for systems code

Example project layout:

project/
├── src/        # Rust / C / C++ code
├── chiru/
│   ├── payment.chiru
│   └── dma_buffer.chiru
└── ci/

Project Status

Chiru v0.1.0

Semantic core complete

Interpreter and verifier implemented

Safety reports (human + JSON)

Stable CLI (check, --help, --version)

Linux binary + installer available

Chiru is experimental but real.

Roadmap

Planned for v0.1.1:

macOS binaries (arm64 + x86_64)

Windows x86_64 binary

Incremental CLI improvements

Policy-driven safety checks

Chiru will grow slowly and deliberately.

Name Policy

The name “Chiru” refers to the official language and tooling maintained by
the chiru-lang organization.

Forks and derivative works must use a different name and must not imply they
are the official Chiru project.

See NAME_POLICY.md for details.

License

Chiru is licensed under the MIT License.

Final Note

Chiru is defined as much by what it refuses to do as by what it allows.

If something reduces trust, Chiru says no.
