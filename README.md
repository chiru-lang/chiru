Chiru

A verification-first systems language for the AI era.

Chiru is a systems programming language designed to make low-level code explicit, inspectable, and trustworthy — especially when code is written or assisted by AI.

Chiru does not replace Rust, C, or C++.
It is designed to live beside them as a verification and trust layer.

TL;DR

🔍 Chiru makes intent explicit

🧠 Chiru produces human-readable safety proofs

🤖 Chiru is designed for AI–human collaboration

⚙️ Chiru targets systems-level code

🧪 Chiru is experimental but real

Why Chiru?

Modern systems programming faces a new trust problem:

AI can generate systems code

Humans struggle to fully audit it

Existing languages rely heavily on inference

Safety is often assumed, not inspected

Chiru exists to answer one question:

How do we trust systems code at scale — especially when AI is involved?

Chiru’s answer:

Make intent explicit

Make safety visible

Make verification inspectable

Make unsafe behavior auditable

What Chiru Is (and Is Not)
Chiru is

A systems-level programming language

Verification-first by design

Explicit about ownership, lifetimes, and capabilities

Deterministic (no garbage collection)

Designed for safety-critical and infrastructure code

Chiru is not

A general-purpose application language

A framework or runtime

A replacement for Rust, C, or C++

A macro-heavy or inference-heavy language

Chiru is intentionally small, strict, and focused.

Core Ideas (At a Glance)

Chiru makes the following first-class and explicit:

Ownership — who owns memory

Lifetimes — how long values are valid

Capabilities — how values may be accessed

Destruction — when values are destroyed

Unsafe assumptions — what humans promise is true

Nothing important is implicit.

Example

    function process_payment {

    lifetime payment_request bound to process_payment

    unsafe {
        assume "HSM returned a valid memory pointer"
        region external hsm_memory
        let payment_key in hsm_memory
    }

    capability SharedRead payment_key during payment_request

    }


This code explicitly states:

where memory comes from

how long it lives

how it may be accessed

what unsafe assumptions are being made

Safety Reports

Every Chiru program produces a Safety Report.

Human-Readable
Chiru Safety Report
==================
    Ownership:     VERIFIED   
    Lifetimes:     VERIFIED
    Capabilities:  VERIFIED
    Destruction:   VERIFIED

Unsafe Assumptions
------------------
    [UA-001] HSM returned a valid memory pointer

Verdict
-------
SAFE_IF_ASSUMPTIONS_HOLD

    Machine-Readable (CI / Automation)
    {
      "verdict": "SAFE_IF_ASSUMPTIONS_HOLD"
    }

Exit Codes
    
    Code	Meaning
    0    	SAFE
    1	    SAFE_IF_ASSUMPTIONS_HOLD
    2	    UNSAFE
    3	    Parse / usage / IO error
    
Installation
Linux (x86_64)

    curl -fsSL https://raw.githubusercontent.com/chiru-lang/chiru/main/install.sh | sh

Verify:

chiru --version

Platform Support
Platform	Status
Linux x86_64	✅ Prebuilt binary
macOS (arm64 / x86_64)	⏳ Build from source
Windows x86_64	⏳ Build from source

Native macOS and Windows binaries are planned for v0.1.1.

Build from Source

Chiru is written in Rust.

    git clone https://github.com/chiru-lang/chiru.git
    cd chiru
    cargo build --release

Binary location:

    target/release/chiru

Where Chiru Fits

Chiru is meant to be used inside larger systems, not to replace them.

Typical use cases:

Verifying AI-generated C / Rust code

Auditing unsafe FFI boundaries

Embedded and firmware verification

Payments, crypto, HSM, security modules

CI safety gates for systems code

Example layout:

    project/
    ├── src/        # Rust / C / C++ code
    ├── chiru/
    │   ├── payment.chiru
    │   └── buffer.chiru
    └── ci/

Project Status

Version: v0.1.0
Status: Experimental but real

Core semantics implemented

Interpreter and verifier working

Safety reports (human + JSON)

Stable CLI

Linux binary available

Roadmap

Planned for v0.1.1:

    macOS binaries
    Windows binaries

Incremental CLI improvements

Policy-driven safety checks

Chiru will evolve slowly and deliberately.

Name & License

License: MIT

Name policy: See NAME_POLICY.md

The name “Chiru” refers to the official language and tooling maintained by
the chiru-lang organization.

Philosophy (Final Word)

Chiru is defined as much by what it refuses to do as by what it allows.

If something reduces trust or auditability, Chiru says no.

